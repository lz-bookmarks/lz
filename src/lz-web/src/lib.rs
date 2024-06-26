use std::io;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::{routing, Router};
use clap::Parser;
use db::GlobalWebAppState;
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::{EnvFilter, Layer as _, Registry};
use utoipa::OpenApi as _;
use utoipa_redoc::{Redoc, Servable as _};
use utoipa_swagger_ui::SwaggerUi;

pub mod api;
pub mod db;
pub mod http;

pub mod export_openapi;

/// The lz tagged bookmark manager web server
#[derive(Clone, Eq, PartialEq, Debug, Parser)]
pub struct Args {
    /// HTTP header that contains the authenticated username.
    #[clap(long)]
    authentication_header_name: String,

    /// Username to use if HTTP headers don't contain one. Useful mainly for tests and dev.
    #[clap(long)]
    default_user_name: Option<String>,

    /// Address to listen on.
    #[clap(long, default_value = "0.0.0.0:8000")]
    listen_on: SocketAddr,
}

pub async fn run(pool: lz_db::Connection, args: &Args) -> anyhow::Result<()> {
    init_observability(args)?;

    let db_conns = Arc::new(GlobalWebAppState::new(
        pool,
        args.authentication_header_name.to_owned(),
        args.default_user_name.to_owned(),
    ));
    let api_router = api::router();
    let app = Router::new()
        .merge(SwaggerUi::new("/docs/swagger").url("/openapi.json", api::ApiDoc::openapi()))
        .merge(Redoc::with_url("/docs/api", api::ApiDoc::openapi()))
        .route("/health", routing::get(health))
        .nest("/api/v1", api_router)
        .layer(sentry_tower::NewSentryLayer::new_from_top())
        .layer(sentry_tower::SentryHttpLayer::with_transaction())
        .with_state(db_conns);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(args.listen_on).await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}

async fn health() -> &'static str {
    "ok"
}

fn init_observability(_args: &Args) -> anyhow::Result<()> {
    // Create a new OpenTelemetry trace pipeline that prints to stdout
    let stderr_log = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_writer(io::stderr)
        .compact()
        .with_filter(EnvFilter::from_default_env());

    // Telemetry for traces and tower-http lives in sentry, if a DSN is configured
    let sentry_layer = sentry_tracing::layer();

    let subscriber = Registry::default().with(stderr_log).with(sentry_layer);
    tracing::subscriber::set_global_default(subscriber).expect("Unable to set global subscriber");

    Ok(())
}
