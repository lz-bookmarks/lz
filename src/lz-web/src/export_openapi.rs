//! An exporter for the current openapi.json file

use std::path::{Path, PathBuf};

use anyhow::Context as _;
use clap::Subcommand;
#[cfg(feature = "dev")]
use progenitor::{GenerationSettings, Generator, InterfaceStyle, TypePatch};
use utoipa::OpenApi as _;

use crate::api;

#[derive(Clone, PartialEq, Eq, Debug, Subcommand)]
pub enum Command {
    /// Generate the rust OpenAPI client for `lz-openapi`.
    #[cfg_attr(not(feature = "dev"), clap(skip))]
    RustClient { crate_root: PathBuf },
    /// Generate the openapi.json spec file.
    Json { out: PathBuf },
}

pub fn run(args: &Command) -> anyhow::Result<()> {
    let json = api::ApiDoc::openapi().to_pretty_json()?;
    match args {
        Command::Json { out } => generate_json(out, &json)?,
        Command::RustClient { crate_root } => generate_rust_client(crate_root, &json)?,
    }
    Ok(())
}

fn generate_json(out: &Path, json: &str) -> anyhow::Result<()> {
    std::fs::write(out, json).with_context(|| format!("writing openapi JSON to {out:?}"))?;
    Ok(())
}

#[cfg(feature = "dev")]
fn generate_rust_client(crate_root: &Path, json: &str) -> anyhow::Result<()> {
    let spec = serde_json::from_str(json).unwrap();
    let mut generator = Generator::new(
        GenerationSettings::new()
            .with_interface(InterfaceStyle::Builder)
            // required by dioxus component props:
            .with_derive("PartialEq")
            // required for hashability:
            .with_derive("Eq")
            .with_derive("Hash")
            // Patch Copy onto all ID types:
            .with_patch("BookmarkId", TypePatch::default().with_derive("Copy"))
            .with_patch("UserId", TypePatch::default().with_derive("Copy")),
    );

    let tokens = generator.generate_tokens(&spec).unwrap();
    let ast = syn::parse2(tokens).unwrap();
    let content = prettyplease::unparse(&ast);

    let out_file = crate_root.join("src/lib.rs").to_path_buf();

    std::fs::write(&out_file, content)
        .with_context(|| format!("writing rust client to {out_file:?}"))?;
    Ok(())
}

#[cfg(not(feature = "dev"))]
fn generate_rust_client(_out: &Path, _json: &str) -> anyhow::Result<()> {
    anyhow::bail!("Not compiled with `--feature dev`, can not generate the necessary rust code.")
}
