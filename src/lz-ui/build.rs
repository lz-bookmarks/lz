use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo::rerun-if-changed=esbuild.js");
    println!("cargo::rerun-if-changed=package.json");
    println!("cargo::rerun-if-changed=yarn.lock");
    println!("cargo::rerun-if-changed=js/");

    Command::new("yarn").args(&["install"]).status()?;
    Command::new("node").args(&["esbuild.js"]).status()?;
    Ok(())
}
