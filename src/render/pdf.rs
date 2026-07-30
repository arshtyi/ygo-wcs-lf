use std::{fs, path::Path};

use anyhow::{Context, Result, bail};
use tempfile::Builder;

use crate::limits::YearLimits;

use super::typst;

pub(super) fn compile(limit_lists: &[YearLimits]) -> Result<()> {
    let project_root = std::env::current_dir().context("failed to resolve project root")?;

    for limits in limit_lists {
        compile_year(limits.year(), &project_root)?;
    }

    Ok(())
}

fn compile_year(year: u16, project_root: &Path) -> Result<()> {
    let output_directory = project_root.join(year.to_string()).join("lf");
    fs::create_dir_all(&output_directory)
        .with_context(|| format!("failed to create {}", output_directory.display()))?;
    let temp = Builder::new()
        .prefix(".ygo-wcs-lf-pdf-")
        .tempdir_in(&output_directory)
        .context("failed to create PDF staging directory")?;
    let source = temp.path().join("lf.typ");
    let staged_pdf = temp.path().join("lf.pdf");
    fs::write(&source, typst_source(year))
        .with_context(|| format!("failed to write Typst entry for {year}"))?;

    let status = typst::command(project_root)?
        .arg(&source)
        .arg(&staged_pdf)
        .current_dir(project_root)
        .status()
        .context("failed to run Typst; ensure `typst` is installed and on PATH")?;
    if !status.success() {
        bail!("Typst failed to compile limit-list PDF for {year}");
    }

    let output = output_directory.join("lf.pdf");
    install_pdf(&staged_pdf, &output, temp.path())?;
    println!("compiled {year} limit-list PDF to {year}/lf/lf.pdf");
    Ok(())
}

fn typst_source(year: u16) -> String {
    format!(
        "#import \"/typst/lf.typ\": render-limit-list\n\n\
         #let limits = json(\"/{year}/data/lf.json\")\n\n\
         #render-limit-list({year}, limits)\n"
    )
}

fn install_pdf(staged: &Path, destination: &Path, temp: &Path) -> Result<()> {
    let backup = temp.join("previous-lf.pdf");
    let had_previous = fs::symlink_metadata(destination).is_ok();

    if had_previous {
        fs::rename(destination, &backup)
            .with_context(|| format!("failed to move existing {}", destination.display()))?;
    }

    if let Err(error) = fs::rename(staged, destination) {
        if had_previous {
            let _ = fs::rename(&backup, destination);
        }
        return Err(error).with_context(|| format!("failed to install {}", destination.display()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::typst_source;

    #[test]
    fn generates_year_pdf_source() {
        let source = typst_source(2026);

        assert!(source.contains("#import \"/typst/lf.typ\": render-limit-list"));
        assert!(source.contains("#let limits = json(\"/2026/data/lf.json\")"));
        assert!(source.contains("#render-limit-list(2026, limits)"));
    }
}
