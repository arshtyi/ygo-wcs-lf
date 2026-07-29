use std::{
    fs::{self, File},
    io::Read,
    path::{Component, Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use flate2::read::GzDecoder;
use tar::Archive;
use xz2::read::XzDecoder;

pub(super) enum Compression {
    Gzip,
    Xz,
}

pub(super) fn extract(
    archive_path: &Path,
    destination: &Path,
    compression: Compression,
) -> Result<()> {
    let source = File::open(archive_path)
        .with_context(|| format!("failed to open {}", archive_path.display()))?;
    let reader: Box<dyn Read> = match compression {
        Compression::Gzip => Box::new(GzDecoder::new(source)),
        Compression::Xz => Box::new(XzDecoder::new(source)),
    };
    let mut archive = Archive::new(reader);

    for entry in archive.entries().context("failed to read tar entries")? {
        let mut entry = entry.context("failed to read tar entry")?;
        let relative = entry.path().context("tar entry has an invalid path")?;
        let relative = safe_relative_path(&relative)?;
        let target = destination.join(&relative);
        let kind = entry.header().entry_type();

        if is_metadata(kind) {
            continue;
        }

        if kind.is_dir() {
            fs::create_dir_all(&target)
                .with_context(|| format!("failed to create {}", target.display()))?;
        } else if kind.is_file() || kind.is_contiguous() {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("failed to create {}", parent.display()))?;
            }
            entry
                .unpack(&target)
                .with_context(|| format!("failed to extract {}", relative.display()))?;
        } else {
            bail!(
                "archive entry type is not allowed: {}",
                relative.display()
            );
        }
    }

    Ok(())
}

fn is_metadata(kind: tar::EntryType) -> bool {
    kind.is_pax_global_extensions()
        || kind.is_pax_local_extensions()
        || kind.is_gnu_longname()
        || kind.is_gnu_longlink()
}

fn safe_relative_path(path: &Path) -> Result<PathBuf> {
    let mut safe = PathBuf::new();

    for component in path.components() {
        match component {
            Component::Normal(part) => safe.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                bail!("unsafe archive path: {}", path.display())
            }
        }
    }

    if safe.as_os_str().is_empty() {
        bail!("archive path is empty");
    }

    Ok(safe)
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{is_metadata, safe_relative_path};

    #[test]
    fn accepts_normal_relative_paths() {
        assert_eq!(
            safe_relative_path(Path::new("./assets/ot/card/ot.json")).unwrap(),
            PathBuf::from("assets/ot/card/ot.json")
        );
    }

    #[test]
    fn rejects_parent_and_absolute_paths() {
        assert!(safe_relative_path(Path::new("../escape")).is_err());
        assert!(safe_relative_path(Path::new("/absolute")).is_err());
    }

    #[test]
    fn allows_only_non_filesystem_metadata_entries() {
        assert!(is_metadata(tar::EntryType::XGlobalHeader));
        assert!(is_metadata(tar::EntryType::XHeader));
        assert!(!is_metadata(tar::EntryType::Symlink));
        assert!(!is_metadata(tar::EntryType::Link));
    }
}
