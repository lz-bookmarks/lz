//! Types to support element interpolation via [HTMZ](https://leanrada.com/htmz/).

use askama_axum::Template;
use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::HeaderMap;
use axum::RequestPartsExt as _;

/// Request argument that indicates what kind of response is expected:
///
/// * Either a full-fledged rendered page, or
/// * A set of elements that need replacing via the HTMZ method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HtmzMode {
    /// User agent requested a stand-alone page
    Standalone,

    /// User agent requested the route from an iframe. This is the HTMZ mode.
    IFrame,
}

#[async_trait]
impl<S> FromRequestParts<S> for HtmzMode {
    /// This never triggers
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let m: HeaderMap = parts.extract::<HeaderMap>().await?;
        if m.get_all("Sec-Fetch-Dest")
            .iter()
            .any(|val| val == "iframe")
        {
            Ok(HtmzMode::IFrame)
        } else {
            Ok(HtmzMode::Standalone)
        }
    }
}

const HTMZ_SCRIPT: &'static str = include_str!("htmz.js");

#[derive(Template, Clone, Debug)]
#[template(path = "auto_htmz.html")]
pub struct HtmzTemplate<T: Template> {
    template: T,
    title: Option<String>,
    mode: HtmzMode,
}

impl HtmzMode {
    pub fn build(self) -> HtmzRenderBuilder {
        HtmzRenderBuilder {
            title: None,
            mode: self,
        }
    }
}

#[derive(Clone, Debug)]
pub struct HtmzRenderBuilder {
    title: Option<String>,
    mode: HtmzMode,
}

impl HtmzRenderBuilder {
    /// Updates the title of the page via htmz.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Finish and wrap a template in a htmz-able template.
    pub fn wrap<T: Template>(self, template: T) -> HtmzTemplate<T> {
        HtmzTemplate {
            template,
            title: self.title,
            mode: self.mode,
        }
    }
}
