//! Functions for interacting with websites that we want to bookmark.

use std::{cell::RefCell, thread_local};

use chrono::Utc;
use lz_db::{Bookmark, NoId};
use scraper::{Html, Selector};

use serde::Serialize;
use url::Url;
use utoipa::ToSchema;

/// Errors that can occur when retrieving content from the web
#[derive(thiserror::Error, Debug)]
pub enum LookupError {
    #[error("could not retrieve link")]
    HttpError(#[from] reqwest::Error),
}

fn make_selector(selector: &str) -> Selector {
    Selector::parse(selector).unwrap()
}

thread_local! {
    static TITLE: RefCell<Selector> = RefCell::new(make_selector("title"));
    static DESCRIPTION: RefCell<Selector> = RefCell::new(make_selector(r#"meta[name="description"]"#));
}

/// Metadata retrieved from a URL
#[derive(PartialEq, Eq, Clone, Debug, Serialize, ToSchema)]
pub struct Metadata {
    /// The title of the document retrieved
    pub title: String,

    /// A description (probably from a meta tag on HTML)
    pub description: Option<String>,
}

/// Retrieves metadata about a link on the web.
pub async fn lookup_page_from_web(url: &Url) -> Result<Metadata, LookupError> {
    tracing::trace!(?url, "retrieving page");
    let client = reqwest::Client::new();
    let response = client.get(url.clone()).send().await?;
    response.error_for_status_ref()?;
    let Ok(body) = response.text().await else {
        return Ok(Metadata {
            title: "Untitled".to_string(),
            description: None,
        });
    };
    let doc = Html::parse_document(&body);
    let root_ref = doc.root_element();
    let found_title = TITLE.with_borrow(|title| root_ref.select(title).next());
    let title = match found_title {
        Some(el) => el.inner_html(),
        None => "Untitled".to_string(),
    };
    let found_description =
        DESCRIPTION.with_borrow(|description| root_ref.select(description).next());
    let description = match found_description {
        Some(el) => el
            .value()
            .attr("content")
            .map(|meta_val| meta_val.to_string()),
        None => None,
    };
    Ok(Metadata { title, description })
}

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
