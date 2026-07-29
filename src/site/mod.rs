mod template;

use std::{
    collections::BTreeSet,
    fs,
    path::Path,
};

use anyhow::{Context, Result, bail};
use tempfile::Builder;

const OUTPUT: &str = "public";
const STYLESHEET: &str = "site/assets/site.css";

struct SiteYear {
    year: u16,
    bytes: u64,
}

pub(crate) fn generate(years: &[u16]) -> Result<()> {
    let root = std::env::current_dir().context("failed to resolve project root")?;
    generate_at(&root, years)
}

fn generate_at(root: &Path, years: &[u16]) -> Result<()> {
    let years = years.iter().copied().collect::<BTreeSet<_>>();
    if years.is_empty() {
        bail!("cannot generate a site without any years");
    }

    let temp = Builder::new()
        .prefix(".ygo-wcs-lf-site-")
        .tempdir_in(root)
        .context("failed to create site staging directory")?;
    let staged = temp.path().join(OUTPUT);
    let assets = staged.join("assets");
    fs::create_dir_all(&assets).context("failed to create staged site assets")?;
    let stylesheet = root.join(STYLESHEET);
    fs::copy(&stylesheet, assets.join("site.css"))
        .with_context(|| format!("failed to copy {}", stylesheet.display()))?;
    fs::write(staged.join(".nojekyll"), []).context("failed to write .nojekyll")?;

    let mut site_years = Vec::with_capacity(years.len());
    for year in years {
        let source = root.join(year.to_string()).join("lf/lf.pdf");
        let metadata = fs::metadata(&source)
            .with_context(|| format!("failed to inspect {}", source.display()))?;
        if !metadata.is_file() {
            bail!("limit-list PDF is not a file: {}", source.display());
        }

        let year_directory = staged.join(year.to_string());
        fs::create_dir(&year_directory)
            .with_context(|| format!("failed to create site directory for {year}"))?;
        fs::copy(&source, year_directory.join("lf.pdf"))
            .with_context(|| format!("failed to copy limit-list PDF for {year}"))?;
        fs::write(
            year_directory.join("index.html"),
            template::viewer(year),
        )
        .with_context(|| format!("failed to write site viewer for {year}"))?;
        site_years.push(SiteYear {
            year,
            bytes: metadata.len(),
        });
    }

    site_years.sort_by_key(|entry| std::cmp::Reverse(entry.year));
    fs::write(staged.join("index.html"), template::index(&site_years))
        .context("failed to write site index")?;
    install(&staged, &root.join(OUTPUT), temp.path())?;

    println!(
        "assembled GitHub Pages site for {} year(s) at {OUTPUT}",
        site_years.len()
    );
    Ok(())
}

fn install(staged: &Path, destination: &Path, temp: &Path) -> Result<()> {
    let backup = temp.join("previous-site");
    let had_previous = fs::symlink_metadata(destination).is_ok();

    if had_previous {
        fs::rename(destination, &backup)
            .with_context(|| format!("failed to move existing {}", destination.display()))?;
    }

    if let Err(error) = fs::rename(staged, destination) {
        if had_previous {
            let _ = fs::rename(&backup, destination);
        }
        return Err(error)
            .with_context(|| format!("failed to install site at {}", destination.display()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::generate_at;

    #[test]
    fn assembles_index_and_viewers() {
        let temp = tempdir().unwrap();
        let stylesheet = temp.path().join("site/assets/site.css");
        fs::create_dir_all(stylesheet.parent().unwrap()).unwrap();
        fs::write(&stylesheet, "body { color: white; }").unwrap();
        for year in [2025, 2026] {
            let directory = temp.path().join(year.to_string()).join("lf");
            fs::create_dir_all(&directory).unwrap();
            fs::write(directory.join("lf.pdf"), format!("pdf-{year}")).unwrap();
        }

        generate_at(temp.path(), &[2025, 2026]).unwrap();

        let index = fs::read_to_string(temp.path().join("public/index.html")).unwrap();
        let year_2025 = index.find("<h2>2025</h2>").unwrap();
        let year_2026 = index.find("<h2>2026</h2>").unwrap();
        assert!(year_2026 < year_2025);
        assert!(index.contains("href=\"2026/index.html\""));
        assert!(!index.contains("download"));

        let viewer =
            fs::read_to_string(temp.path().join("public/2026/index.html")).unwrap();
        assert!(viewer.contains("src=\"./lf.pdf#view=FitH\""));
        assert!(!viewer.contains("<header"));
        assert!(!viewer.contains("download"));
        assert_eq!(
            fs::read(temp.path().join("public/2025/lf.pdf")).unwrap(),
            b"pdf-2025"
        );
        assert!(temp.path().join("public/assets/site.css").is_file());
        assert_eq!(
            fs::read_to_string(temp.path().join("public/assets/site.css")).unwrap(),
            "body { color: white; }"
        );
        assert!(!temp.path().join("public/assets/site.js").exists());
        assert!(temp.path().join("public/.nojekyll").is_file());
    }

    #[test]
    fn rejects_missing_pdf() {
        let temp = tempdir().unwrap();

        assert!(generate_at(temp.path(), &[2026]).is_err());
        assert!(!temp.path().join("public").exists());
    }
}
