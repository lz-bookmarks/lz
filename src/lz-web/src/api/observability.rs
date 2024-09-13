//! Layers and middleware that improve understanding of what's going on in the API

use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, Response};
use axum::Router;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::{debug, field, info_span, Span};

pub(super) fn add_layers<T: Clone + Send + Sync + 'static>(router: Router<T>) -> Router<T> {
    router.layer(
        ServiceBuilder::new().layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<Body>| {
                    info_span!(
                        "http request",
                        // Fields on the request that interest us:
                        "request.method" = %request.method(),
                        "request.uri" = %request.uri(),
                        "request.http_version" = ?request.version(),
                        "request.headers" = ?request.headers(),

                        // Fields that get filled later:
                        "response.status_code" = field::Empty,
                        "response.status" = field::Empty,
                        "response.latency_ms" = field::Empty,
                        "response.headers" = field::Empty,
                    )
                })
                .on_response(
                    |response: &Response<Body>, latency: Duration, span: &Span| {
                        span.record(
                            "status_code",
                            tracing::field::display(response.status().as_u16()),
                        );
                        span.record("response.status", field::display(response.status()));
                        span.record(
                            "response.status_code",
                            field::display(response.status().as_u16()),
                        );
                        span.record("response.headers", field::debug(response.headers()));
                        span.record("response.latency_ms", latency.as_millis());
                        debug!("response generated")
                    },
                ),
        ),
    )
}
