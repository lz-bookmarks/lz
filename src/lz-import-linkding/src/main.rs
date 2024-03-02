use std::path::PathBuf;

use clap::Parser;

#[derive(Clone, Debug, PartialEq, Eq, Parser)]
struct Args {
    /// The linkding backup .sqlite3 file to import
    linkding_backup: PathBuf,

    /// The lz database to import into
    #[clap(long)]
    to: PathBuf,

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
    .await?;
    let linkding_tx =
        lz_import_linkding::schema::LinkdingTransaction::from_pool(&mut linkding_db).await?;

    let lz_db_pool = sqlx::Pool::connect(&format!("sqlite:{}", args.to.to_string_lossy())).await?;
    let lz = lz_db::Connection::from_pool(lz_db_pool);
    let mut tx = lz.begin_for_user(&args.user).await?;
    lz_import_linkding::migrate::migrate(linkding_tx, &mut tx).await?;
    tx.commit().await?;
    Ok(())
}
