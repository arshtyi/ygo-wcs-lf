use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result, bail};
use tempfile::Builder;

use crate::limits::YearLimits;

const TYPST: &str = "typst";
const PREVIEWS: &str = "build/previews";
const TYPST_YGO: &str = "vendor/typst-ygo";

pub(super) fn compile(limit_lists: &[YearLimits], ppi: u16) -> Result<()> {
    if !(36..=144).contains(&ppi) {
        bail!("preview PPI must be between 36 and 144");
    }

    let identifiers = ordered_card_ids(limit_lists);
    if identifiers.is_empty() {
        bail!("no card IDs found to render");
    }

    let project_root = std::env::current_dir().context("failed to resolve project root")?;
    let workspace = project_root.join(TYPST_YGO);
    validate_workspace(&workspace)?;

    let build = project_root.join("build");
    fs::create_dir_all(&build).context("failed to create build directory")?;
    let temp = Builder::new()
        .prefix(".ygo-wcs-lf-render-")
        .tempdir_in(&build)
        .context("failed to create preview staging directory")?;
    let source = temp.path().join("ot.typ");
    fs::write(&source, typst_source(&identifiers))
        .context("failed to write generated card Typst source")?;
    let staged = temp.path().join("previews/ot");
    fs::create_dir_all(&staged).context("failed to create staged preview directory")?;
    let output_pattern = staged.join("page-{0p}.png");

    let mut command = Command::new(TYPST);
    command
        .arg("compile")
        .arg("--root")
        .arg(&project_root);
    for font_path in font_paths(&workspace)? {
        command.arg("--font-path").arg(font_path);
    }
    let status = command
        .arg("--ppi")
        .arg(ppi.to_string())
        .arg(&source)
        .arg(&output_pattern)
        .current_dir(&project_root)
        .status()
        .context("failed to run Typst; ensure `typst` is installed and on PATH")?;

    if !status.success() {
        bail!("Typst failed to render card previews");
    }

    let pages = rendered_pages(&staged)?;
    if pages.len() != identifiers.len() {
        bail!(
            "Typst rendered {} pages for {} cards",
            pages.len(),
            identifiers.len()
        );
    }
    for (page, identifier) in pages.iter().zip(&identifiers) {
        fs::rename(page, staged.join(format!("{identifier}.png")))
            .with_context(|| format!("failed to name preview for card {identifier}"))?;
    }

    install_previews(&temp.path().join("previews"), &project_root.join(PREVIEWS), temp.path())?;
    println!(
        "rendered {} card previews at {ppi} PPI to {PREVIEWS}/ot",
        identifiers.len()
    );
    Ok(())
}

fn ordered_card_ids(limit_lists: &[YearLimits]) -> Vec<u32> {
    let mut seen = HashSet::new();
    limit_lists
        .iter()
        .flat_map(YearLimits::ids)
        .filter(|id| seen.insert(*id))
        .collect()
}

fn typst_source(identifiers: &[u32]) -> String {
    let values = identifiers
        .iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(", ");

    format!(
        "#import \"/vendor/typst-ygo/lib/mod.typ\": ot_card_by_id, ot_card_data\n\n\
         #let cards = ot_card_data()\n\
         #let ids = ({values},)\n\n\
         #for id in ids {{ ot_card_by_id(id, cards: cards) }}\n"
    )
}

fn rendered_pages(directory: &Path) -> Result<Vec<PathBuf>> {
    let mut pages = fs::read_dir(directory)
        .with_context(|| format!("failed to inspect {}", directory.display()))?
        .map(|entry| {
            let path = entry?.path();
            let page = page_number(&path)?;
            Ok((page, path))
        })
        .collect::<Result<Vec<_>>>()?;
    pages.sort_by_key(|(page, _)| *page);
    Ok(pages.into_iter().map(|(_, path)| path).collect())
}

fn page_number(path: &Path) -> Result<u32> {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .context("preview filename is not valid UTF-8")?;
    file_name
        .strip_prefix("page-")
        .and_then(|name| name.strip_suffix(".png"))
        .context("unexpected preview filename")?
        .parse()
        .with_context(|| format!("invalid preview page number in {file_name}"))
}

fn font_paths(workspace: &Path) -> Result<[PathBuf; 2]> {
    let paths = [
        workspace.join("assets/ot/font"),
        workspace.join("assets/rd/font"),
    ];
    for path in &paths {
        if !path.is_dir() {
            bail!("required Typst font directory is missing: {}", path.display());
        }
    }
    Ok(paths)
}

fn validate_workspace(workspace: &Path) -> Result<()> {
    let required = [
        workspace.join("lib/mod.typ"),
        workspace.join("assets/ot/card/ot.json"),
    ];
    for path in required {
        if !path.is_file() {
            bail!(
                "required typst-ygo file is missing: {}; run `prepare` first",
                path.display()
            );
        }
    }
    Ok(())
}

fn install_previews(staged: &Path, destination: &Path, temp: &Path) -> Result<()> {
    let backup = temp.join("previous-previews");
    let had_previous = fs::symlink_metadata(destination).is_ok();

    if had_previous {
        fs::rename(destination, &backup).with_context(|| {
            format!(
                "failed to move existing previews {}",
                destination.display()
            )
        })?;
    }

    if let Err(error) = fs::rename(staged, destination) {
        if had_previous {
            let _ = fs::rename(&backup, destination);
        }
        return Err(error)
            .with_context(|| format!("failed to install previews at {}", destination.display()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{page_number, typst_source};

    #[test]
    fn generates_ot_card_source() {
        let source = typst_source(&[89_631_139, 23_427_709]);

        assert!(source.contains(
            "#import \"/vendor/typst-ygo/lib/mod.typ\": ot_card_by_id, ot_card_data"
        ));
        assert!(source.contains("#let ids = (89631139, 23427709,)"));
        assert!(source.contains("ot_card_by_id(id, cards: cards)"));
    }

    #[test]
    fn parses_preview_page_numbers() {
        assert_eq!(page_number(Path::new("page-1.png")).unwrap(), 1);
        assert_eq!(page_number(Path::new("page-0012.png")).unwrap(), 12);
        assert!(page_number(Path::new("card-1.png")).is_err());
    }
}
