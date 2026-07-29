use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use tempfile::Builder;

use super::{
    archive::{self, Compression},
    download::Downloader,
};

const TYPST_YGO_URL: &str =
    "https://github.com/arshtyi/typst-ygo/archive/refs/heads/main.tar.gz";
const ASSETS_URL: &str =
    "https://github.com/arshtyi/ygo-assets/releases/download/latest/assets.tar.xz";
const OT_CARDS_URL: &str =
    "https://github.com/arshtyi/ygo-cards/releases/download/latest/ot.json";
const WORKSPACE: &str = "vendor/typst-ygo";

pub(super) async fn assemble(downloader: &Downloader) -> Result<PathBuf> {
    let destination = PathBuf::from(WORKSPACE);
    let parent = destination
        .parent()
        .context("workspace destination has no parent")?;
    fs::create_dir_all(parent)
        .with_context(|| format!("failed to create {}", parent.display()))?;
    let temp = Builder::new()
        .prefix(".ygo-wcs-lf-")
        .tempdir_in(parent)
        .context("failed to create workspace staging directory")?;
    let downloads = temp.path().join("downloads");
    fs::create_dir(&downloads).context("failed to create download staging directory")?;

    let typst_archive = downloads.join("typst-ygo.tar.gz");
    let assets_archive = downloads.join("assets.tar.xz");
    let ot_cards = downloads.join("ot.json");

    println!("downloading typst-ygo, card data, and card assets...");
    tokio::try_join!(
        downloader.download(TYPST_YGO_URL, &typst_archive),
        downloader.download(ASSETS_URL, &assets_archive),
        downloader.download(OT_CARDS_URL, &ot_cards),
    )?;

    let typst_extract = temp.path().join("typst-extract");
    let assets_extract = temp.path().join("assets-extract");
    fs::create_dir(&typst_extract).context("failed to create typst extraction directory")?;
    fs::create_dir(&assets_extract).context("failed to create assets extraction directory")?;
    archive::extract(&typst_archive, &typst_extract, Compression::Gzip)?;
    archive::extract(&assets_archive, &assets_extract, Compression::Xz)?;

    let upstream_root = single_directory(&typst_extract)?;
    let staged = temp.path().join("typst-ygo");
    fs::rename(&upstream_root, &staged).context("failed to stage typst-ygo template")?;

    let extracted_assets = assets_root(&assets_extract)?;
    copy_tree_contents(&extracted_assets, &staged.join("assets"))?;

    let staged_cards = staged.join("assets/ot/card/ot.json");
    if let Some(parent) = staged_cards.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::copy(&ot_cards, &staged_cards).context("failed to stage OT card data")?;

    preserve_center_images(&destination, &staged)?;
    validate_workspace(&staged)?;
    install(&staged, &destination, temp.path())?;

    println!("prepared typst-ygo workspace at {}", destination.display());
    Ok(destination)
}

fn preserve_center_images(previous: &Path, staged: &Path) -> Result<()> {
    let previous_images = previous.join("assets/ot/images");
    if previous_images.is_dir() {
        copy_tree_contents(&previous_images, &staged.join("assets/ot/images"))?;
    }
    Ok(())
}

fn assets_root(extracted: &Path) -> Result<PathBuf> {
    if extracted.join("assets").is_dir() {
        return Ok(extracted.join("assets"));
    }
    if extracted.join("ot").is_dir() {
        return Ok(extracted.to_owned());
    }
    single_directory(extracted)
}

fn single_directory(path: &Path) -> Result<PathBuf> {
    let mut directories = Vec::new();
    let mut has_files = false;

    for entry in
        fs::read_dir(path).with_context(|| format!("failed to inspect {}", path.display()))?
    {
        let entry = entry?;
        let kind = entry.file_type()?;
        if kind.is_dir() {
            directories.push(entry.path());
        } else {
            has_files = true;
        }
    }

    if directories.len() != 1 || has_files {
        bail!(
            "expected one archive root directory in {}",
            path.display()
        );
    }

    Ok(directories.remove(0))
}

fn copy_tree_contents(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination)
        .with_context(|| format!("failed to create {}", destination.display()))?;

    for entry in
        fs::read_dir(source).with_context(|| format!("failed to inspect {}", source.display()))?
    {
        let entry = entry?;
        let kind = entry.file_type()?;
        let target = destination.join(entry.file_name());

        if kind.is_dir() {
            copy_tree_contents(&entry.path(), &target)?;
        } else if kind.is_file() {
            fs::copy(entry.path(), &target)
                .with_context(|| format!("failed to copy {}", entry.path().display()))?;
        } else {
            bail!(
                "workspace source links are not allowed: {}",
                entry.path().display()
            );
        }
    }

    Ok(())
}

fn validate_workspace(workspace: &Path) -> Result<()> {
    let required = [
        workspace.join("lib/mod.typ"),
        workspace.join("assets/ot/images"),
        workspace.join("assets/ot/card/ot.json"),
    ];
    let missing = required
        .iter()
        .filter(|path| !path.exists())
        .map(|path| {
            path.strip_prefix(workspace)
                .unwrap_or(path)
                .display()
                .to_string()
        })
        .collect::<Vec<_>>();

    if !missing.is_empty() {
        bail!("upstream workspace is incomplete: {}", missing.join(", "));
    }

    Ok(())
}

fn install(staged: &Path, destination: &Path, temp: &Path) -> Result<()> {
    let backup = temp.join("previous-workspace");
    let had_previous = destination.exists();

    if had_previous {
        fs::rename(destination, &backup).with_context(|| {
            format!(
                "failed to move existing workspace {}",
                destination.display()
            )
        })?;
    }

    if let Err(error) = fs::rename(staged, destination) {
        if had_previous {
            let _ = fs::rename(&backup, destination);
        }
        return Err(error).with_context(|| {
            format!("failed to install workspace at {}", destination.display())
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::{copy_tree_contents, preserve_center_images, single_directory};

    #[test]
    fn finds_single_archive_root() {
        let temp = tempdir().unwrap();
        let root = temp.path().join("root");
        fs::create_dir(&root).unwrap();

        assert_eq!(single_directory(temp.path()).unwrap(), root);
    }

    #[test]
    fn copies_directory_contents_recursively() {
        let temp = tempdir().unwrap();
        let source = temp.path().join("source");
        let destination = temp.path().join("destination");
        fs::create_dir_all(source.join("nested")).unwrap();
        fs::write(source.join("nested/file.txt"), "value").unwrap();

        copy_tree_contents(&source, &destination).unwrap();

        assert_eq!(
            fs::read_to_string(destination.join("nested/file.txt")).unwrap(),
            "value"
        );
    }

    #[test]
    fn preserves_existing_center_images() {
        let temp = tempdir().unwrap();
        let previous = temp.path().join("previous");
        let staged = temp.path().join("staged");
        fs::create_dir_all(previous.join("assets/ot/images")).unwrap();
        fs::create_dir_all(staged.join("assets/ot/images")).unwrap();
        fs::write(
            previous.join("assets/ot/images/123.jpg"),
            [0xff, 0xd8],
        )
        .unwrap();

        preserve_center_images(&previous, &staged).unwrap();

        assert_eq!(
            fs::read(staged.join("assets/ot/images/123.jpg")).unwrap(),
            [0xff, 0xd8]
        );
    }
}
