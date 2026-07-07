use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Create a new cargo-factorio project in the current directory.
    Init(InitArgs),
    /// Transpile Rust sources to Lua.
    Build(BuildArgs),
}

#[derive(Debug, Parser)]
#[command(
    name = "factorio",
    bin_name = "cargo factorio",
    about = "Transpile Rust into Lua for Factorio mods",
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Parser)]
pub struct InitArgs {
    /// Name of the generated Cargo package.
    #[arg(long, value_name = "NAME")]
    pub name: Option<String>,

    /// Path to the project directory or `Factorio.toml` file.
    #[arg(long, value_name = "PATH")]
    pub manifest_path: Option<PathBuf>,
}

#[derive(Debug, Parser)]
pub struct BuildArgs {
    /// Path to the project directory or `Factorio.toml` file.
    #[arg(long, value_name = "PATH")]
    pub manifest_path: Option<PathBuf>,
}
