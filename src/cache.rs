use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::{Serialize, de::DeserializeOwned};
use tempfile::NamedTempFile;

const CACHE_ROOT: &str = ".cache/ygo-wcs-lf";

pub(crate) fn path(relative: impl AsRef<Path>) -> PathBuf {
    Path::new(CACHE_ROOT).join(relative)
}

pub(crate) fn read_json<T>(path: &Path) -> Result<Option<T>>
where
    T: DeserializeOwned,
{
    let json = match fs::read(path) {
        Ok(json) => json,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(error).with_context(|| format!("failed to read {}", path.display()));
        }
    };
    let value = serde_json::from_slice(&json)
        .with_context(|| format!("invalid cache data in {}", path.display()))?;

    Ok(Some(value))
}

pub(crate) fn write_json<T>(path: &Path, value: &T) -> Result<()>
where
    T: Serialize,
{
    let parent = path.parent().context("cache path has no parent")?;
    fs::create_dir_all(parent)
        .with_context(|| format!("failed to create cache directory {}", parent.display()))?;
    let mut temporary = NamedTempFile::new_in(parent)
        .with_context(|| format!("failed to create cache file in {}", parent.display()))?;

    serde_json::to_writer_pretty(&mut temporary, value)
        .with_context(|| format!("failed to encode cache {}", path.display()))?;
    temporary
        .write_all(b"\n")
        .with_context(|| format!("failed to write cache {}", path.display()))?;
    temporary
        .as_file()
        .sync_all()
        .with_context(|| format!("failed to flush cache {}", path.display()))?;
    temporary
        .persist(path)
        .map_err(|error| error.error)
        .with_context(|| format!("failed to replace cache {}", path.display()))?;

    Ok(())
}
