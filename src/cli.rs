use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(version, about)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    /// Resolve forbidden and limited card lists for one or more years.
    Lf {
        /// One or more World Championship years.
        #[arg(required = true, num_args = 1..)]
        years: Vec<u16>,
    },
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::{Cli, Command};

    #[test]
    fn parses_lf_subcommand_with_multiple_years() {
        let cli = Cli::try_parse_from(["ygo-wcs-lf", "lf", "2025", "2026"]).unwrap();

        assert!(matches!(
            cli.command,
            Command::Lf { years } if years == [2025, 2026]
        ));
    }

    #[test]
    fn requires_at_least_one_year() {
        assert!(Cli::try_parse_from(["ygo-wcs-lf", "lf"]).is_err());
    }
}
