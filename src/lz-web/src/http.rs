//! Functions for interacting with websites that we want to bookmark.

use chrono::Utc;
use lz_db::{Bookmark, NoId};

pub use lz_http::*;
use url::Url;

/// Retrieve metadata for a link and pre-fill a [Bookmark] structure
pub async fn lookup_bookmark_from_web(url: &Url) -> Result<Bookmark<NoId, NoId>, LookupError> {
    let now = Utc::now();
    let Metadata { title, description } = lookup_page_from_web(url).await?;
    let to_add = Bookmark {
        accessed_at: Some(now),
        created_at: now,
        description: description.clone(),
        id: NoId,
        import_properties: None,
        modified_at: None,
        notes: None,
        shared: true,
        title: title.clone(),
        unread: true,
        url: url.clone(),
        user_id: NoId,
        website_title: if title.as_str() == "" {
            None
        } else {
            Some(title.clone())
        },
        website_description: description.clone(),
    };
    Ok(to_add)
}
