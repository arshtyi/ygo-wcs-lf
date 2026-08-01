use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};

pub(crate) const TEMPLATE_DIRECTORY: &str = "vendor/typst-ygo";
const TEMPLATE_SOURCE: &str = "upstream/typst-ygo";
const ASSETS_SOURCE: &str = "upstream/ygo-assets/assets";

pub(crate) struct Upstream {
    template_source: PathBuf,
    assets_source: PathBuf,
    workspace: PathBuf,
}

impl Upstream {
    pub(crate) fn at(project_root: &Path) -> Self {
        Self {
            template_source: project_root.join(TEMPLATE_SOURCE),
            assets_source: project_root.join(ASSETS_SOURCE),
            workspace: project_root.join(TEMPLATE_DIRECTORY),
        }
    }

    pub(crate) fn template(&self) -> &Path {
        &self.workspace
    }

    pub(crate) fn card_data(&self) -> PathBuf {
        self.workspace.join("assets/ot/card/ot.json")
    }

    pub(crate) fn center_images(&self) -> PathBuf {
        self.workspace.join("assets/ot/images")
    }

    pub(crate) fn font_directories(&self) -> [PathBuf; 2] {
        [
            self.workspace.join("assets/ot/font"),
            self.workspace.join("assets/rd/font"),
        ]
    }

    pub(crate) fn prepare_layout(&self) -> Result<()> {
        self.validate_sources()?;

        fs::create_dir_all(self.workspace.join("assets/ot")).with_context(|| {
            format!("failed to create workspace at {}", self.workspace.display())
        })?;
        ensure_directory_link(
            &self.template_source.join("lib"),
            &self.workspace.join("lib"),
        )?;
        ensure_directory_link(
            &self.assets_source.join("rd"),
            &self.workspace.join("assets/rd"),
        )?;

        let ot_source = self.assets_source.join("ot");
        for entry in fs::read_dir(&ot_source)
            .with_context(|| format!("failed to inspect {}", ot_source.display()))?
        {
            let entry = entry?;
            let name = entry.file_name();
            if name == "card" || name == "images" {
                continue;
            }
            if !entry.file_type()?.is_dir() {
                bail!("unexpected asset source file: {}", entry.path().display());
            }
            ensure_directory_link(&entry.path(), &self.workspace.join("assets/ot").join(name))?;
        }

        fs::create_dir_all(self.workspace.join("assets/ot/card"))
            .context("failed to create card data workspace")?;
        fs::create_dir_all(self.workspace.join("assets/ot/images"))
            .context("failed to create center image workspace")
    }

    pub(crate) fn validate_ready(&self) -> Result<()> {
        self.validate_sources()?;

        let required = [self.card_data(), self.center_images()];
        let missing = required
            .iter()
            .filter(|path| !path.exists())
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>();
        if !missing.is_empty() {
            bail!(
                "upstream resources are incomplete: {}; run `prepare` first",
                missing.join(", ")
            );
        }

        Ok(())
    }

    fn validate_sources(&self) -> Result<()> {
        let required = [
            self.template_source.join("lib/mod.typ"),
            self.assets_source.join("ot/font"),
            self.assets_source.join("rd/font"),
            self.assets_source.join("ot/card"),
            self.assets_source.join("ot/images"),
        ];
        let missing = required
            .iter()
            .filter(|path| !path.exists())
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>();
        if !missing.is_empty() {
            bail!(
                "upstream submodules are incomplete: {}; run `git submodule update --init`",
                missing.join(", ")
            );
        }

        Ok(())
    }
}

fn ensure_directory_link(source: &Path, link: &Path) -> Result<()> {
    if let Ok(metadata) = fs::symlink_metadata(link) {
        if metadata.file_type().is_symlink() && same_target(link, source)? {
            return Ok(());
        }
        bail!(
            "{} must be a link to {}; remove it before preparing resources",
            link.display(),
            source.display()
        );
    }

    create_directory_link(source, link)
        .with_context(|| format!("failed to link {} to {}", link.display(), source.display()))
}

fn same_target(link: &Path, expected: &Path) -> Result<bool> {
    let actual = fs::canonicalize(link)
        .with_context(|| format!("failed to resolve asset link {}", link.display()))?;
    let expected = fs::canonicalize(expected)
        .with_context(|| format!("failed to resolve asset directory {}", expected.display()))?;
    Ok(actual == expected)
}

#[cfg(unix)]
fn create_directory_link(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(windows)]
fn create_directory_link(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_dir(target, link)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::Upstream;

    #[test]
    fn links_template_to_versioned_assets() {
        let temp = tempdir().unwrap();
        let upstream = scaffold(temp.path());

        upstream.prepare_layout().unwrap();
        upstream.prepare_layout().unwrap();

        assert_eq!(
            fs::canonicalize(upstream.template().join("lib")).unwrap(),
            fs::canonicalize(temp.path().join("upstream/typst-ygo/lib")).unwrap()
        );
        assert_eq!(
            fs::canonicalize(upstream.template().join("assets/ot/font")).unwrap(),
            fs::canonicalize(temp.path().join("upstream/ygo-assets/assets/ot/font")).unwrap()
        );
        assert_eq!(
            fs::canonicalize(upstream.template().join("assets/rd")).unwrap(),
            fs::canonicalize(temp.path().join("upstream/ygo-assets/assets/rd")).unwrap()
        );
        assert!(upstream.template().join("assets/ot/card").is_dir());
        assert!(upstream.template().join("assets/ot/images").is_dir());
    }

    #[test]
    fn rejects_conflicting_asset_directory() {
        let temp = tempdir().unwrap();
        let upstream = scaffold(temp.path());
        fs::create_dir_all(upstream.template()).unwrap();
        fs::create_dir(upstream.template().join("lib")).unwrap();

        assert!(upstream.prepare_layout().is_err());
    }

    fn scaffold(root: &std::path::Path) -> Upstream {
        let upstream = Upstream::at(root);
        fs::create_dir_all(root.join("upstream/typst-ygo/lib")).unwrap();
        fs::write(root.join("upstream/typst-ygo/lib/mod.typ"), "").unwrap();
        for directory in ["ot/font", "ot/frame", "rd/font", "ot/card", "ot/images"] {
            fs::create_dir_all(root.join("upstream/ygo-assets/assets").join(directory)).unwrap();
        }
        upstream
    }
}
