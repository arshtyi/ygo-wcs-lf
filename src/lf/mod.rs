mod model;
mod parser;

use std::{fs, path::PathBuf};

use anyhow::{Context, Result, bail};

pub(crate) fn run(year: u16) -> Result<()> {
    let input_path = PathBuf::from(year.to_string()).join("data/lf.list");
    let source = fs::read_to_string(&input_path)
        .with_context(|| format!("failed to read {}", input_path.display()))?;
    let parsed = parser::parse(&source);

    for diagnostic in &parsed.diagnostics {
        eprintln!(
            "{}:{}: skipped: {}",
            input_path.display(),
            diagnostic.line_number,
            diagnostic.message
        );
    }

    bail!(
        "parsed {} cards; ID resolution is not implemented yet",
        parsed.cards.len()
    )
}
