use std::path::PathBuf;
use std::str::FromStr;

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Local, LocalResult, NaiveDateTime, TimeZone, Utc};
use clap::{Parser, Subcommand};
use lz_db::{BookmarkId, BookmarkSearch, Connection, DateInput, ReadOnly, Transaction};
use std::collections::HashSet;
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
    #[clap(long, global = true, default_value = "db.sqlite")]
    db: PathBuf,
}

#[derive(Parser, Debug)]
struct TuiArgs {
    /// User name to operate on.
    #[clap(long, default_value = "local")]
    user: String,
}

/// A date timestamp specified as 0:00:00 local time.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
struct LocalDatestamp(DateTime<Utc>);

impl FromStr for LocalDatestamp {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // user_created_at is a string and should be in 'YYYY-MM-DD` format.
        let dt = NaiveDateTime::parse_from_str(&format!("{} 00:00:00", s), "%Y-%m-%d %H:%M:%S")?;
        let LocalResult::Single(local_time) = Local.from_local_datetime(&dt) else {
            anyhow::bail!("Could not convert naive datestamp {dt:?} to local DateTime")
        };
        Ok(LocalDatestamp(local_time.to_utc()))
    }
}

#[derive(Parser, Debug)]
struct CliAddArgs {
    /// The URL to add
    link: String,
    /// Assign a date other than today for the link's creation (in
    /// YYYY-MM-DD format)
    #[arg(long)]
    backdate: Option<LocalDatestamp>,
    /// User-provided description for the link
    #[arg(long)]
    description: Option<String>,
    /// Overwrite values for any existing bookmarks
    #[arg(long, action)]
    force: bool,
    /// Freeform notes for this link
    #[arg(long)]
    notes: Option<String>,
    /// Tag (or tags as a comma-delineated list) for the link
    #[arg(short, long, value_delimiter = ',', num_args = 1..)]
    tag: Vec<String>,
    /// User-provided title for the link
    #[arg(long)]
    title: Option<String>,
    /// Add a subordinated link for this bookmark
    #[arg(long)]
    associated_link: Option<String>,
    /// Optional context for the association
    #[arg(long)]
    associated_context: Option<String>,
}

#[derive(Subcommand, Debug)]
enum ImportCommands {
    /// Import from linkding (https://github.com/sissbruecker/linkding)
    Linkding(lz_import_linkding::Args),
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Add a link to lz
    Add {
        #[clap(flatten)]
        common_args: TuiArgs,
        #[clap(flatten)]
        add_args: CliAddArgs,
    },

    /// Delete a link from lz
    #[clap(alias = "rm")]
    Remove {
        #[clap(flatten)]
        common_args: TuiArgs,
        /// The URL to remove.
        link: String,
    },

    /// List bookmarks
    #[clap(alias = "ls")]
    List {
        #[clap(flatten)]
        common_args: TuiArgs,

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

    /// Add or remove tags from existing bookmarks
    Tag {
        #[clap(flatten)]
        common_args: TuiArgs,
        /// Bookmark for which the tags should be updated
        link: String,
        /// Tag (or tags as a comma-delineated list) for the link
        #[arg(short, long, value_delimiter = ',', num_args = 1..)]
        tag: Vec<String>,
        /// Delete, rather than add, these tags
        #[arg(action, long, short)]
        delete: bool,
    },

    /// Run the lz web server
    #[clap(alias = "serve")]
    Web(lz_web::Args),

    /// Import bookmarks from another system
    #[clap(subcommand)]
    Import(ImportCommands),

    /// Writes the contents of the openapi.json file to stdout
    #[clap(subcommand, alias = "generate-openapi-spec")]
    GenerateOpenApiSpec(lz_web::export_openapi::Command),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Add {
            common_args,
            add_args,
        } => {
            let conn = Connection::from_path(&cli.db).await?;
            let mut txn = conn.begin_for_user(&common_args.user).await?;
            add_cmd(&mut txn, add_args).await?;
            txn.commit().await?;
        }
        Commands::List {
            common_args,
            created_after,
            created_before,
            tagged,
        } => {
            let conn = Connection::from_path(&cli.db).await?;
            let txn = conn.begin_ro_for_user(&common_args.user).await?;
            list_cmd(txn, created_after, created_before, tagged).await?;
        }
        Commands::Remove { common_args, link } => {
            let conn = Connection::from_path(&cli.db).await?;
            let mut txn = conn.begin_for_user(&common_args.user).await?;
            remove_cmd(&mut txn, link).await?;
            txn.commit().await?;
        }
        Commands::Tag {
            common_args,
            link,
            delete,
            tag,
        } => {
            let conn = Connection::from_path(&cli.db).await?;
            let mut txn = conn.begin_for_user(&common_args.user).await?;
            tag_cmd(&mut txn, link, tag, delete).await?;
            txn.commit().await?;
        }
        Commands::Web(args) => {
            let conn = Connection::from_path(&cli.db).await?;
            lz_web::run(conn, args).await?;
        }
        Commands::Import(ImportCommands::Linkding(args)) => {
            let conn = Connection::from_path(&cli.db).await?;
            lz_import_linkding::run(conn, args).await?;
        }
        Commands::GenerateOpenApiSpec(args) => lz_web::export_openapi::run(args)?,
    }
    Ok(())
}

async fn list_cmd(
    mut txn: Transaction<ReadOnly>,
    created_after: &Option<String>,
    created_before: &Option<String>,
    tagged: &Option<Vec<String>>,
) -> Result<()> {
    let mut last_seen = None;
    let page_size = 1000;

    // All datetimes currently use the sqlite3 `localtime` options; for purposes of
    // dates, we'll eventually want to allow a config option setting a default timezone.
    let mut filters: Vec<BookmarkSearch> = vec![];
    if let Some(created_before_str) = created_before {
        let dt = created_before_str.parse::<DateInput>()?;
        filters.push(lz_db::created_before_from_datetime(dt));
    };
    if let Some(created_after_str) = created_after {
        let dt = created_after_str.parse::<DateInput>()?;
        filters.push(lz_db::created_after_from_datetime(dt))
    };
    if let Some(tag_strings) = tagged {
        for namestring in tag_strings.iter() {
            let tag = lz_db::TagName(namestring.clone());
            filters.push(BookmarkSearch::TagByName { tag });
        }
    }
    loop {
        let bookmarks = txn
            .list_bookmarks_matching(&filters, page_size, last_seen)
            .await?;
        for (elt, bm) in bookmarks.iter().enumerate() {
            if elt == usize::from(page_size) {
                last_seen = Some(bm.id);
                break;
            }
            println!("{}: <{}>", bm.title, bm.url);
        }
        if bookmarks.len() < usize::from(page_size) + 1 {
            return Ok(());
        }
    }
}

async fn add_cmd(txn: &mut Transaction, args: &CliAddArgs) -> Result<()> {
    let bookmark_id = add_link(
        txn,
        args.link.to_string(),
        args.backdate.as_ref(),
        args.description.as_deref(),
        args.force,
        args.notes.as_deref(),
        args.title.as_deref(),
    )
    .await?;
    if !args.tag.is_empty() {
        let mut deduped = args.tag.clone();
        deduped.sort_unstable();
        deduped.dedup();
        let tags = txn.ensure_tags(&deduped).await?;
        txn.set_bookmark_tags(bookmark_id, tags).await?;
    }
    if let Some(associate) = &args.associated_link {
        // Associations don't get user-configurable notes, description, etc.
        // If the user wants this, they should go add the associate as a primary
        // link and then associate separately.
        let associated_url = Url::parse(associate)?;
        let url_id = txn.ensure_url(&associated_url).await?;
        txn.associate_bookmark_link(&bookmark_id, &url_id, args.associated_context.as_deref())
            .await?;
    }
    println!("Added bookmark for <{}>", args.link);
    Ok(())
}

async fn add_link(
    txn: &mut Transaction,
    link: String,
    backdate: Option<&LocalDatestamp>,
    description: Option<&str>,
    force: bool,
    notes: Option<&str>,
    title: Option<&str>,
) -> Result<BookmarkId> {
    let url = Url::parse(&link).with_context(|| format!("invalid url {:?}", link))?;
    let mut bookmark = lz_web::http::lookup_link_from_web(&url).await?;
    if let Some(user_title) = title {
        bookmark.title = user_title.to_string();
    }
    if let Some(user_description) = description {
        bookmark.description = Some(user_description.to_string());
    }
    if let Some(user_created_at) = backdate {
        bookmark.created_at = user_created_at.0;
    }
    bookmark.notes = notes.map(|n| n.to_string());
    match txn.add_bookmark(bookmark.clone()).await {
        Ok(v) => Ok(v),
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
            if force {
                // Known to be valid or we would have errored out on the URL parse
                let url = Url::parse(&link).unwrap();
                let mut existing_bookmark = txn.find_bookmark_with_url(&url).await?.unwrap();
                existing_bookmark.description = bookmark.description;
                existing_bookmark.notes = bookmark.notes;
                existing_bookmark.title = bookmark.title;
                existing_bookmark.website_description = bookmark.website_description;
                existing_bookmark.website_title = bookmark.website_title;
                match txn.update_bookmark(&existing_bookmark).await {
                    Ok(_) => Ok(existing_bookmark.id),
                    Err(err) => Err(err.into()),
                }
            } else {
                Err(anyhow!(
                    "<{}> is already bookmarked; use --force to override",
                    &link
                ))
            }
        }
        Err(err) => Err(err.into()),
    }
}

async fn remove_cmd(txn: &mut Transaction, link: &String) -> Result<()> {
    let url = Url::parse(link).with_context(|| format!("invalid url {:?}", link))?;
    let existing_bookmark = txn.find_bookmark_with_url(&url).await?;
    if let Some(bookmark) = existing_bookmark {
        let result = txn.delete_bookmark(bookmark.id).await?;
        if result {
            println!("Removed <{}>", link);
        } else {
            println!("Unable to remove <{}>", link);
        }
    } else {
        println!("<{}> not found", link);
    }
    Ok(())
}

async fn tag_cmd(
    txn: &mut Transaction,
    link: &String,
    tag: &Vec<String>,
    delete: &bool,
) -> Result<()> {
    if tag.is_empty() {
        println!("Tag or tags required");
        return Ok(());
    } else {
        let url = Url::parse(link).with_context(|| format!("invalid url {:?}", link))?;
        let existing_bookmark = txn.find_bookmark_with_url(&url).await?;
        if let Some(bookmark) = existing_bookmark {
            let existing_tags = txn.get_bookmark_tags(bookmark.id).await?;
            if *delete {
                let deletable: Vec<_> = tag.iter().map(lz_db::normalize_tag).collect();
                let mut new_tags: Vec<_> = existing_tags;
                new_tags.retain(|t| !deletable.contains(&t.name));
                txn.set_bookmark_tags(bookmark.id, new_tags).await?;
            } else {
                let mut new_tags = txn.ensure_tags(tag).await?;
                new_tags.extend(existing_tags);
                let mut seen = HashSet::new();
                new_tags.retain(|t| {
                    let unique = !seen.contains(&t.id);
                    seen.insert(t.id);
                    unique
                });
                txn.set_bookmark_tags(bookmark.id, new_tags).await?;
            }
            println!("<{}> tags updated", link);
        } else {
            println!("<{}> not found", link);
        }
    }
    Ok(())
}
