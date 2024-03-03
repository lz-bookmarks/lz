use std::path::PathBuf;

use anyhow::Context as _;
use clap::Parser;

#[derive(Clone, Debug, PartialEq, Eq, Parser)]
struct Args {
    /// The linkding backup .sqlite3 file to import
    linkding_backup: PathBuf,

    /// The lz database to import into
    #[clap(long)]
    to: PathBuf,

    /// What to do when encountering a bookmark under the same URL
    #[clap(long, default_value_t, value_enum)]
    on_duplicate: lz_import_linkding::DuplicateBehavior,

    /// The username on the lz side to import as.
    #[clap(long)]
    user: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    let mut linkding_db = sqlx::Pool::connect(&format!(
        "sqlite:{}",
        args.linkding_backup.to_string_lossy()
    ))
    .await
    .with_context(|| format!("origin DB file {}", args.linkding_backup.to_string_lossy()))?;
    let linkding_tx =
        lz_import_linkding::schema::LinkdingTransaction::from_pool(&mut linkding_db).await?;
    let lz_db_pool = sqlx::Pool::connect(&format!("sqlite:{}", args.to.to_string_lossy()))
        .await
        .with_context(|| format!("destination DB file {}", args.to.to_string_lossy()))?;
    let lz = lz_db::Connection::from_pool(lz_db_pool);
    let tx = lz.begin_for_user(&args.user).await?;
    let migration = lz_import_linkding::Migration::new(tx, linkding_tx, args.on_duplicate);
    migration.migrate().await?;
    Ok(())
}
