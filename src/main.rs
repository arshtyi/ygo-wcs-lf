mod cli;
mod lf;
mod prepare;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Lf { years } => lf::run(years).await,
        Command::Prepare { years } => prepare::run(years).await,
    }
}
