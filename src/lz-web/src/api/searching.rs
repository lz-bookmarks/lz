//! Types and operations to support searching for items via the web API.

use std::sync::Arc;

use axum::{
    extract::{rejection::PathRejection, FromRequestParts, Path},
    http::{request::Parts, StatusCode},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::{ToResponse, ToSchema};

use crate::db::GlobalWebAppState;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, ToSchema, ToResponse)]
pub(crate) struct TagName(#[schema(min_length = 1, pattern = "^[^ ]+$")] String);

impl AsRef<str> for TagName {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

/// A search query for retrieving bookmarks via the tags assigned to them.
///
/// These tag queries are made in a URL path, separated by space
/// (`%20`) characters.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, ToSchema, ToResponse)]
pub(crate) struct TagQuery {
    /// Tags that all returned items should have.
    #[schema(min_items = 1)]
    pub tags: Vec<TagName>,
}

#[derive(Error, Debug)]
pub enum TagQueryRejection {
    #[error("Invalid format on `{0}`")]
    InvalidTagFormat(String),

    #[error("Internal error: The handler has no `*query` parameter defined")]
    NoStarQueryParameterDefined(#[from] PathRejection),
}

impl IntoResponse for TagQueryRejection {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::BAD_REQUEST, self.to_string()).into_response()
    }
}

#[async_trait::async_trait]
impl FromRequestParts<Arc<GlobalWebAppState>> for TagQuery {
    type Rejection = TagQueryRejection;
    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<GlobalWebAppState>,
    ) -> Result<Self, Self::Rejection> {
        let Path(query): Path<String> = Path::from_request_parts(parts, state).await?;
        let tags = query
            .split(' ')
            .filter(|q| !q.is_empty())
            .map(|q| TagName(q.to_string()))
            .collect();
        Ok(TagQuery { tags })
    }
}
