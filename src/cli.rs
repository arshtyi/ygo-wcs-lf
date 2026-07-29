use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(version, about)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    /// Run the complete limit-list build for selected or discovered years.
    Build {
        /// World Championship years; discovers all years when omitted.
        #[arg(num_args = 0..)]
        years: Vec<u16>,
    },
    /// Resolve forbidden and limited card lists for one or more years.
    Lf {
        /// One or more World Championship years.
        #[arg(required = true, num_args = 1..)]
        years: Vec<u16>,
    },
    /// Download resources and sort limit lists for one or more years.
    Prepare {
        /// One or more World Championship years.
        #[arg(required = true, num_args = 1..)]
        years: Vec<u16>,
    },
    /// Render card previews and limit-list PDFs for one or more years.
    Render {
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
    fn parses_build_subcommand_with_multiple_years() {
        let cli = Cli::try_parse_from(["ygo-wcs-lf", "build", "2025", "2026"]).unwrap();

        assert!(matches!(
            cli.command,
            Command::Build { years } if years == [2025, 2026]
        ));
    }

    #[test]
    fn allows_build_subcommand_without_years() {
        let cli = Cli::try_parse_from(["ygo-wcs-lf", "build"]).unwrap();

        assert!(matches!(
            cli.command,
            Command::Build { years } if years.is_empty()
        ));
    }

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

    #[test]
    fn parses_prepare_subcommand_with_multiple_years() {
        let cli = Cli::try_parse_from(["ygo-wcs-lf", "prepare", "2025", "2026"]).unwrap();

        assert!(matches!(
            cli.command,
            Command::Prepare { years } if years == [2025, 2026]
        ));
    }

    #[test]
    fn parses_render_subcommand_with_multiple_years() {
        let cli = Cli::try_parse_from(["ygo-wcs-lf", "render", "2025", "2026"]).unwrap();

        assert!(matches!(
            cli.command,
            Command::Render { years } if years == [2025, 2026]
        ));
    }
}
