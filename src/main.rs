mod build;
mod cli;
mod lf;
mod limits;
mod prepare;
mod render;
mod years;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Build { years } => build::run(years).await,
        Command::Lf { years } => lf::run(years).await,
        Command::Prepare { years } => prepare::run(years).await,
        Command::Render { years } => render::run(years),
    }
}
