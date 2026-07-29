use anyhow::{Context, Result};

use crate::{lf, prepare, render, years};

pub(crate) async fn run(years: Vec<u16>) -> Result<()> {
    let root = std::env::current_dir().context("failed to resolve project root")?;
    let years = years::resolve(&root, years)?;

    println!(
        "building World Championship limit lists for {}",
        years::display(&years)
    );
    lf::run(years.clone()).await?;
    prepare::run(years.clone()).await?;
    render::run(years)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use crate::years;

    #[test]
    fn build_year_discovery_uses_limit_list_directories() {
        let temp = tempdir().unwrap();
        for year in ["2026", "2025"] {
            fs::create_dir_all(temp.path().join(year).join("data")).unwrap();
            fs::write(
                temp.path().join(year).join("data/lf.list"),
                "/// ocg\n/// forbidden\nA // A",
            )
            .unwrap();
        }

        assert_eq!(years::resolve(temp.path(), Vec::new()).unwrap(), [2025, 2026]);
    }
}
