use std::{net::SocketAddr, path::PathBuf, sync::Arc};

use axum::Router;

use clap::Parser;
use lz_web::db::GlobalWebAppState;

use utoipa::OpenApi as _;
use utoipa_redoc::{Redoc, Servable as _};
use utoipa_swagger_ui::SwaggerUi;

/// The lz tagged bookmark manager web server
#[derive(Clone, Eq, PartialEq, Debug, Parser)]
struct Args {
    /// SQLite database file where we keep bookmarks
    #[clap(long)]
    db: PathBuf,

    /// HTTP header that contains the authenticated username.
    #[clap(long)]
    authentication_header_name: String,

    /// Username to use if HTTP headers don't contain one. Useful mainly for tests.
    #[clap(long)]
    default_user_name: Option<String>,

    /// Address to listen on.
    #[clap(long, default_value = "0.0.0.0:8000")]
    listen_on: SocketAddr,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    let args = Args::parse();

    let pool =
        lz_db::Connection::from_pool(sqlx::SqlitePool::connect(&args.db.to_string_lossy()).await?);
    let db_conns = Arc::new(GlobalWebAppState::new(
        pool,
        args.authentication_header_name,
        args.default_user_name,
    ));
    let api_router = lz_web::api::router();
    let app = Router::new()
        .merge(
            SwaggerUi::new("/docs/swagger")
                .url("/docs/openapi.json", lz_web::api::ApiDoc::openapi()),
        )
        .merge(Redoc::with_url("/docs/api", lz_web::api::ApiDoc::openapi()))
        .nest("/api/v1", api_router)
        .with_state(db_conns);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(args.listen_on).await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
