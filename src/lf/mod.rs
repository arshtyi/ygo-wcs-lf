mod api;
mod model;
mod output;
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

    if parsed.cards.is_empty() {
        bail!("no valid cards found in {}", input_path.display());
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

    if resolved.cards.is_empty() {
        bail!(
            "no card IDs could be resolved; {} was not written",
            input_path.with_file_name("lf.json").display()
        );
    }

    let output_path = input_path.with_file_name("lf.json");
    output::write(&output_path, &resolved.cards)?;

    let skipped = parsed.diagnostics.len() + resolved.diagnostics.len();
    println!(
        "wrote {} card IDs to {} ({skipped} skipped)",
        resolved.cards.len(),
        output_path.display()
    );

    Ok(())
}
