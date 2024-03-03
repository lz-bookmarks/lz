use serde::{Deserialize, Serialize};

pub mod schema;

pub mod migrate;
pub use migrate::Migration;

/// What to do about duplicate bookmarks.
///
/// The user can choose to either overwrite or duplicate each
/// already-existing bookmark for a URL.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Default, Deserialize, Serialize, clap::ValueEnum)]
#[serde(rename = "camel_case")]
pub enum DuplicateBehavior {
    Overwrite,

    #[default]
    Skip,
}
