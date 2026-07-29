use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(version, about)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    /// Resolve a year's forbidden and limited card lists.
    Lf {
        /// World Championship year.
        year: u16,
    },
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::{Cli, Command};

    #[test]
    fn parses_lf_subcommand() {
        let cli = Cli::try_parse_from(["ygo-wcs-lf", "lf", "2026"]).unwrap();

        assert!(matches!(cli.command, Command::Lf { year: 2026 }));
    }
}
