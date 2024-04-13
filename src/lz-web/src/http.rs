//! Functions for interacting with websites that we want to bookmark.

use chrono::Utc;
use lazy_static::lazy_static;
use lz_db::{Bookmark, NoId};
use scraper::{Html, Selector};
use url::Url;

/// Errors that can occur when retrieving content from the web
#[derive(thiserror::Error, Debug)]
pub enum LookupError {
    #[error("could not retrieve link")]
    HttpError(#[from] reqwest::Error),
}

fn make_selector(selector: &str) -> Selector {
    Selector::parse(selector).unwrap()
}

lazy_static! {
    static ref TITLE: Selector = make_selector("title");
    static ref DESCRIPTION: Selector = make_selector(r#"meta[name="description"]"#);
}

pub async fn lookup_link_from_web(url: &Url) -> Result<Bookmark<NoId, NoId>, LookupError> {
    // This currently assumes all lookups are against HTML pages, which is a
    // reasonable starting point but would prevent e.g. bookmarking images.
    let now = Utc::now();
    let response = reqwest::get(url.clone()).await?;
    response.error_for_status_ref()?;
    let body = response.text().await?;
    let doc = Html::parse_document(&body);
    let root_ref = doc.root_element();
    let found_title = root_ref.select(&TITLE).next();
    let title = match found_title {
        Some(el) => el.inner_html(),
        None => "".to_string(),
    };
    let found_description = root_ref.select(&DESCRIPTION).next();
    let description = match found_description {
        Some(el) => el
            .value()
            .attr("content")
            .map(|meta_val| meta_val.to_string()),
        None => None,
    };
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
