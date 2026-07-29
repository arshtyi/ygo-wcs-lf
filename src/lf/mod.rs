mod api;
mod model;
mod parser;
mod resolver;

use std::{fs, path::PathBuf};

use anyhow::{Context, Result, bail};

use self::api::ApiClient;

pub(crate) async fn run(year: u16) -> Result<()> {
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

    let api = ApiClient::new()?;
    let resolved = resolver::resolve(&parsed.cards, &api).await;

    for diagnostic in &resolved.diagnostics {
        eprintln!(
            "{}:{}: skipped: {}",
            input_path.display(),
            diagnostic.line_number,
            diagnostic.message
        );
    }

    bail!(
        "resolved {} cards; JSON output is not implemented yet",
        resolved.cards.len()
    )
}
