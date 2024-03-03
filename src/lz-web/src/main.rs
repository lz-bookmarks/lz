use axum::{routing::get, Router};

use utoipa::OpenApi;
use utoipa_rapidoc::RapiDoc;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    #[derive(OpenApi)]
    #[openapi(paths(root))]
    struct ApiDoc;

    // build our application with a route
    let app = Router::new()
        .merge(RapiDoc::with_openapi("/api-docs/openapi2.json", ApiDoc::openapi()).path("/rapidoc"))
        // `GET /` goes to `root`
        .route("/", get(root));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

/// basic handler that responds with a static string
#[utoipa::path(get, path = "/")]
async fn root() -> &'static str {
    "Hello, World!"
}
