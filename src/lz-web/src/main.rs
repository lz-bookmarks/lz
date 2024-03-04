use std::{net::SocketAddr, path::PathBuf, sync::Arc};

use axum::{routing::get, Router};

use clap::Parser;
use lz_web::db::{DbTransaction, GlobalWebAppState};
use utoipa::OpenApi;
use utoipa_rapidoc::RapiDoc;

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
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    #[derive(OpenApi)]
    #[openapi(paths(root, lz_web::api::list_bookmarks))]
    struct ApiDoc;

    let pool =
        lz_db::Connection::from_pool(sqlx::SqlitePool::connect(&args.db.to_string_lossy()).await?);
    let db_conns = Arc::new(GlobalWebAppState {
        pool,
        authentication_header_name: args.authentication_header_name,
        default_user_name: args.default_user_name,
    });
    let app = Router::new()
        .merge(RapiDoc::with_openapi("/api-docs/openapi2.json", ApiDoc::openapi()).path("/rapidoc"))
        // `GET /` goes to `root`
        .route("/", get(root))
        .nest("/api", lz_web::api::router())
        .with_state(db_conns);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(args.listen_on).await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}

/// basic handler that responds with a static string
#[utoipa::path(get, path = "/",
    responses(
        (status = 200, description = "Hello world."),
    ),
)]
#[tracing::instrument()]
async fn root(txn: DbTransaction) -> &'static str {
    tracing::info!(user = ?(*txn).user().id);
    "Hello, World!"
}
