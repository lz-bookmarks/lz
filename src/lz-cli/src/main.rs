use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use chrono::NaiveDateTime;
use clap::{Parser, Subcommand};
use lz_db::{Bookmark, BookmarkId, BookmarkSearch, Connection, Transaction};
use scraper::{Html, Selector};
use url::Url;

// NB See https://rust-cli-recommendations.sunshowers.io/handling-arguments.html for
// advice  on structuring the subcommands
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Path to the database to use
    #[clap(long, default_value = "db.sqlite")]
    db: PathBuf,

    #[clap(long, default_value = "local")]
    user: String,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Add a link to lz
    Add {
        /// The URL to add
        link: String,
        /// User-provided description for the link
        #[arg(long)]
        description: Option<String>,
        /// Freeform notes for this link
        #[arg(long)]
        notes: Option<String>,
        /// Tag (or tags as a comma-delineated list) for the link
        #[arg(short, long, value_delimiter = ',', num_args = 1..)]
        tag: Option<Vec<String>>,
        /// User-provided title for the link
        #[arg(long)]
        title: Option<String>,
    },
    /// List bookmarks
    #[clap(alias = "ls")]
    List {
        /// Created on or after a date; accepts a YYYY-MM-DD string
        #[arg(long)]
        created_after: Option<String>,
        /// Created before a date; accepts a YYYY-MM-DD string
        #[arg(long)]
        created_before: Option<String>,
        /// Tag (or tags as a comma-delineated list) for the link; note
        /// that listed bookmarks must be tagged with all tags given.
        #[arg(long, value_delimiter = ',', num_args = 1..)]
        tagged: Option<Vec<String>>,
    },
}

#[tokio::main]
async fn main() {
    if let Err(e) = _main().await {
        eprintln!("Error: {:#?}", e);
        std::process::exit(1)
    }
}

async fn _main() -> Result<()> {
    let cli = Cli::parse();
    let pool =
        sqlx::sqlite::SqlitePool::connect(&format!("sqlite:{}", cli.db.to_string_lossy())).await?;
    let conn = Connection::from_pool(pool);
    let txn = conn.begin_for_user(&cli.user).await?;

    match &cli.command {
        Commands::Add {
            link,
            description,
            notes,
            tag,
            title,
        } => {
            add_cmd(txn, link, description, notes, tag, title).await?;
        }
        Commands::List {
            created_after,
            created_before,
            tagged,
        } => {
            list_cmd(txn, created_after, created_before, tagged).await?;
        }
    }
    Ok(())
}

fn datestring_to_datetime(
    datestring: String,
    end_of_day: bool,
) -> Result<chrono::DateTime<chrono::Utc>> {
    let naive = if end_of_day {
        NaiveDateTime::parse_from_str(&format!("{} 23:59:59", &datestring), "%Y-%m-%d %H:%M:%S")
    } else {
        NaiveDateTime::parse_from_str(&format!("{} 00:00:00", &datestring), "%Y-%m-%d %H:%M:%S")
    };
    if let Ok(naive) = naive {
        Ok(naive.and_utc())
    } else {
        Err(anyhow!(
            "{} is not a valid YYYY-MM-DD date string",
            datestring
        ))
    }
}

async fn list_cmd(
    mut txn: Transaction,
    created_after: &Option<String>,
    created_before: &Option<String>,
    tagged: &Option<Vec<String>>,
) -> Result<()> {
    let mut last_seen = None;
    let page_size = 1000;

    // All datetimes are UTC; for purposes of dates, we'll eventually want to allow a config
    // option setting a default timezone.
    let mut filters: Vec<BookmarkSearch> = vec![];
    if let Some(created_before_str) = created_before {
        let dt = datestring_to_datetime(created_before_str.to_string(), false)?;
        filters.push(lz_db::created_before_from_datetime(dt))
    };
    if let Some(created_after_str) = created_after {
        let dt = datestring_to_datetime(created_after_str.to_string(), false)?;
        filters.push(lz_db::created_after_from_datetime(dt))
    };
    if let Some(tag_strings) = tagged {
        for namestring in tag_strings.iter() {
            let name = lz_db::TagName(namestring.clone());
            filters.push(BookmarkSearch::TagByName { name });
        }
    }
    loop {
        let bookmarks = txn
            .list_bookmarks_matching(filters.clone(), page_size, last_seen)
            .await?;
        for (elt, bm) in bookmarks.iter().enumerate() {
            if elt == usize::from(page_size) {
                last_seen = Some(bm.id);
                break;
            }
            println!("{}: {}", bm.title, bm.url);
        }
        if bookmarks.len() < usize::from(page_size) + 1 {
            return Ok(());
        }
    }
}

async fn add_cmd(
    mut txn: Transaction,
    link: &String,
    description: &Option<String>,
    notes: &Option<String>,
    tag: &Option<Vec<String>>,
    title: &Option<String>,
) -> Result<()> {
    let bookmark_id = add_link(&mut txn, link.to_string(), description, notes, title).await?;
    if let Some(tag_strings) = tag {
        let tags = txn.ensure_tags(tag_strings).await?;
        txn.set_bookmark_tags(bookmark_id, tags).await?;
    }
    txn.commit().await?;
    Ok(())
}

async fn add_link(
    txn: &mut Transaction,
    link: String,
    description: &Option<String>,
    notes: &Option<String>,
    title: &Option<String>,
) -> Result<BookmarkId> {
    let mut bookmark = lookup_link_from_web(link).await?;
    if let Some(user_title) = title {
        bookmark.title = user_title.to_string();
    }
    if let Some(user_description) = description {
        bookmark.description = Some(user_description.to_string());
    }
    bookmark.notes = notes.as_ref().map(|n| n.to_string());
    match txn.add_bookmark(bookmark.clone()).await {
        Ok(v) => Ok(v),
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
            // TODO:
            // This is recoverable -- what's the right thing to do when the
            // user adds an existing tag?
            // I guess the sensible thing to do here as we abstract this into a
            // shared library is potentially support a force flag, which could
            // be set from the caller based on context? But e.g. in the web client
            // what should we do with the new description etc.?
            Err(db_err.into())
        }
        Err(err) => Err(err.into()),
    }
}

async fn lookup_link_from_web(link: String) -> Result<Bookmark<(), ()>> {
    // This currently assumes all lookups are against HTML pages, which is a
    // reasonable starting point but would prevent e.g. bookmarking images.
    let now = chrono::Utc::now();
    let url = Url::parse(&link).context("Invalid link")?;
    let response = reqwest::get(link).await?;
    response.error_for_status_ref()?;
    let body = response.text().await?;
    let doc = Html::parse_document(&body);
    let root_ref = doc.root_element();
    let found_title = root_ref.select(&Selector::parse("title").unwrap()).next();
    let title = match found_title {
        Some(el) => el.inner_html(),
        None => "".to_string(),
    };
    let found_description = root_ref
        .select(&Selector::parse(r#"meta[name="description"]"#).unwrap())
        .next();
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
        id: (),
        import_properties: None,
        modified_at: None,
        notes: None,
        shared: true,
        title: title.clone(),
        unread: true,
        url,
        user_id: (),
        website_title: if title.as_str() == "" {
            None
        } else {
            Some(title.clone())
        },
        website_description: description.clone(),
    };
    Ok(to_add)
}
