use std::path::PathBuf;

use anyhow::Context as _;
use clap::Parser;
use serde::{Deserialize, Serialize};

pub mod schema;

pub mod migrate;
pub use migrate::Migration;

/// Arguments for the linkding import
#[derive(Clone, Debug, PartialEq, Eq, Parser)]
pub struct Args {
    /// The linkding backup .sqlite3 file to import
    linkding_backup: PathBuf,

    /// What to do when encountering a bookmark under the same URL
    #[clap(long, default_value_t, value_enum)]
    on_duplicate: DuplicateBehavior,

    /// The username on the lz side to import as.
    #[clap(long)]
    user: String,
}

/// Start the import from the linkding database
pub async fn run(lz: lz_db::Connection, args: &Args) -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let mut linkding_db = sqlx::Pool::connect(&format!(
        "sqlite:{}",
        args.linkding_backup.to_string_lossy()
    ))
    .await
    .with_context(|| format!("origin DB file {}", args.linkding_backup.to_string_lossy()))?;
    let linkding_tx = schema::LinkdingTransaction::from_pool(&mut linkding_db).await?;
    let tx = lz.begin_for_user(&args.user).await?;
    let migration = Migration::new(tx, linkding_tx, args.on_duplicate);
    migration.migrate().await?;
    Ok(())
}

/// What to do about duplicate bookmarks.
///
/// The user can choose to either overwrite or duplicate each
/// already-existing bookmark for a URL.
#[derive(
    Clone, PartialEq, Copy, Eq, Hash, Debug, Default, Deserialize, Serialize, clap::ValueEnum,
)]
#[serde(rename = "camel_case")]
pub enum DuplicateBehavior {
    Overwrite,

    #[default]
    Skip,
}
