//! Types and operations to support searching for items via the web API.

use std::sync::Arc;

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum_extra::extract::{Query, QueryRejection};
use lz_db::TagName;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::{IntoParams, ToResponse, ToSchema};

use crate::db::GlobalWebAppState;

/// A search query for retrieving bookmarks via the tags assigned to them.
///
/// Each
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, IntoParams, ToSchema, ToResponse)]
#[into_params(style = Form, parameter_in = Query)]
pub(crate) struct TagQuery {
    /// Tags that all returned items should have.
    #[schema(min_items = 1)]
    #[serde(default)]
    pub tags: Vec<TagName>,
}

#[derive(Error, Debug)]
pub enum TagQueryRejection {
    #[error("Internal error: The handler has no `*query` parameter defined")]
    NoStarQueryParameterDefined(#[from] QueryRejection),
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
        let Query(q) = Query::from_request_parts(parts, state).await?;
        Ok(q)
    }
}
