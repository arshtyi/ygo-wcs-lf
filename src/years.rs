use std::{collections::BTreeSet, fs, path::Path};

use anyhow::{Context, Result, bail};

pub(crate) fn resolve(root: &Path, years: Vec<u16>) -> Result<Vec<u16>> {
    if !years.is_empty() {
        return Ok(years
            .into_iter()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect());
    }

    discover(root)
}

pub(crate) fn display(years: &[u16]) -> String {
    years
        .iter()
        .map(u16::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

fn discover(root: &Path) -> Result<Vec<u16>> {
    let mut years = BTreeSet::new();

    for entry in
        fs::read_dir(root).with_context(|| format!("failed to inspect {}", root.display()))?
    {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        let Ok(year) = name.parse::<u16>() else {
            continue;
        };
        if entry.path().join("data/lf.list").is_file() {
            years.insert(year);
        }
    }

    if years.is_empty() {
        bail!(
            "no World Championship years found; expected [year]/data/lf.list under {}",
            root.display()
        );
    }

    Ok(years.into_iter().collect())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::{display, resolve};

    #[test]
    fn normalizes_explicit_years() {
        assert_eq!(
            resolve(tempdir().unwrap().path(), vec![2026, 2025, 2026]).unwrap(),
            [2025, 2026]
        );
    }

    #[test]
    fn discovers_sorted_years_with_limit_lists() {
        let temp = tempdir().unwrap();
        for path in ["2026/data/lf.list", "2025/data/lf.list"] {
            let path = temp.path().join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, "").unwrap();
        }
        fs::create_dir(temp.path().join("2024")).unwrap();
        fs::create_dir(temp.path().join("notes")).unwrap();

        assert_eq!(resolve(temp.path(), Vec::new()).unwrap(), [2025, 2026]);
    }

    #[test]
    fn rejects_empty_discovery() {
        let temp = tempdir().unwrap();

        assert!(resolve(temp.path(), Vec::new()).is_err());
    }

    #[test]
    fn formats_years() {
        assert_eq!(display(&[2025, 2026]), "2025, 2026");
    }
}
