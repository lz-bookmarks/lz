use std::path::PathBuf;

use clap::Parser;

#[derive(Clone, Debug, PartialEq, Eq, Parser)]
struct Args {
    linkding_backup: PathBuf,
}

fn main() {}
