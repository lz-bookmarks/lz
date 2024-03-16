use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use lz_db::{Bookmark, BookmarkId, Connection, Transaction};
use reqwest;
use scraper::{Html, Selector};
use sqlx;
use url::Url;

// NB See https://rust-cli-recommendations.sunshowers.io/handling-arguments.html for
// advice  on structuring the subcommands
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Add a link to lz
    Add {
        /// The URL to add
        link: String,
        #[arg(short, long, value_delimiter = ',', num_args = 1..)]
        /// Tag (or tags as a comma-delineated list) for the link
        tag: Option<Vec<String>>,
    },
    /// List bookmarks
    #[clap(alias = "ls")]
    List {},
}

#[tokio::main]
async fn main() {
    if let Err(e) = _main().await {
        eprintln!("Error: {:#?}", e);
        std::process::exit(1)
    }
}

// TODO: --title to support title
// TODO: --description
// TODO: --notes
// tag subcommand
// TODO: Config file for location of sqlite3 file
// TODO: Handle creation of the database if it doesn't exist
// TODO: Custom error message on duplicate URL
// TODO: --force tagg on `add` to support upsert

async fn _main() -> Result<()> {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Add { link, tag } => {
            let pool = sqlx::sqlite::SqlitePool::connect("sqlite:lz.db").await?;
            let conn = Connection::from_pool(pool);
            let mut txn = conn.begin_for_user("local").await?;
            let bookmark_id = add(&mut txn, link.to_string()).await?;
            if let Some(tag_strings) = tag {
                let tags = txn.ensure_tags(tag_strings).await?;
                txn.set_bookmark_tags(bookmark_id, tags).await?;
            }
            txn.commit().await?;
        },
        Commands::List {} => {
            list_bookmarks().await?;
        }
    }
    Ok(())
}

async fn list_bookmarks() -> Result<()> {
    let pool = sqlx::sqlite::SqlitePool::connect("sqlite:lz.db").await?;
    let conn = Connection::from_pool(pool);
    let mut txn = conn.begin_for_user("local").await?;
    let bookmarks = txn.all_bookmarks(None).await?;
    for i in &bookmarks {
        println!("{}: {}", i.title, i.url);
    }
    Ok(())
}

async fn add(txn: &mut Transaction, link: String) -> Result<BookmarkId> {
    let bookmark = lookup_link(link).await?;
    match txn.add_bookmark(bookmark.clone()).await {
        Ok(v) => {
            Ok(v)
        },
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
            println!("Duplicate link");
            // TODO:
            // This is recoverable -- what's the right thing to do when the
            // user adds an existing tag?
            // I guess the sensible thing to do here as we abstract this into a
            // shared library is potentially support a force flag, which could
            // be set from the caller based on context? But e.g. in the web client
            // what should we do with the new description etc.?
            Err(db_err.into())
        },
        Err(err) => Err(err.into()),
    }
}

async fn lookup_link(link: String) -> Result<Bookmark<(), ()>> {
    let url = Url::parse(&link).context("Invalid link")?;
    let response = reqwest::get(link).await?;
    response.error_for_status_ref()?;
    let body = response.text().await?;
    let doc =  Html::parse_document(&body);
    let root_ref = doc.root_element();
    let title = root_ref.select(&Selector::parse("title").unwrap())
        .next()
        .context("Unable to find title")
        .unwrap()
        .inner_html();
    // FIXME: Can we get rid of the clones, here?
    // TODO: Handle blank title.
    // TODO: Allow overriding title
    let to_add = Bookmark {
        accessed_at: None,
        created_at: Default::default(),
        description: None,
        id: (),
        import_properties: None,
        modified_at: None,
        notes: None,
        shared: true,
        title: title.clone(),
        unread: true,
        url: url,
        user_id: (),
        website_title: Some(title.clone()),
        website_description: None,
    };
    Ok(to_add)
}
