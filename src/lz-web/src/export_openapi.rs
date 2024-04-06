//! An exporter for the current openapi.json file

use clap::Parser;
use utoipa::OpenApi as _;

use crate::api;

#[derive(Clone, PartialEq, Eq, Debug, Parser)]
pub struct Args {}

pub fn run() -> anyhow::Result<()> {
    println!("{}", api::ApiDoc::openapi().to_pretty_json()?);
    Ok(())
}
