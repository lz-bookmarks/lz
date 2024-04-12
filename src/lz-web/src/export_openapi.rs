//! An exporter for the current openapi.json file

use std::path::PathBuf;

use anyhow::Context as _;
use clap::Parser;
#[cfg(feature = "dev")]
use progenitor::{GenerationSettings, Generator, InterfaceStyle, TypePatch};
use utoipa::OpenApi as _;

use crate::api;

#[derive(Clone, PartialEq, Eq, Debug, Parser)]
pub struct Args {
    #[cfg_attr(feature = "dev", clap(long))]
    #[cfg(feature = "dev")]
    rust_client: Option<PathBuf>,
    #[clap(long)]
    openapi_json: Option<PathBuf>,
}

pub fn run(args: &Args) -> anyhow::Result<()> {
    let json = api::ApiDoc::openapi().to_pretty_json()?;
    if let Some(out) = &args.openapi_json {
        std::fs::write(out, &json).with_context(|| format!("writing {out:?}"))?;
    }
    #[cfg(feature = "dev")]
    if let Some(dest_crate) = &args.rust_client {
        let spec = serde_json::from_str(&json).unwrap();
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

        let out_file = dest_crate.join("src/lib.rs").to_path_buf();

        std::fs::write(&out_file, content)
            .with_context(|| format!("writing rust client to {out_file:?}"))?;
    }
    Ok(())
}
