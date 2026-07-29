use std::{
    fs,
    path::Path,
};

use anyhow::{Context, Result, bail};

use super::SiteYear;

const INDEX: &str = "site/index.html";
const YEAR_CARD: &str = "site/year-card.html";
const VIEWER: &str = "site/viewer.html";

pub(super) struct Templates {
    index: String,
    year_card: String,
    viewer: String,
}

impl Templates {
    pub(super) fn load(root: &Path) -> Result<Self> {
        Ok(Self {
            index: read(root, INDEX)?,
            year_card: read(root, YEAR_CARD)?,
            viewer: read(root, VIEWER)?,
        })
    }

    pub(super) fn index(&self, years: &[SiteYear]) -> Result<String> {
        let cards = years
            .iter()
            .map(|entry| {
                render(
                    &self.year_card,
                    &[
                        ("year", entry.year.to_string()),
                        ("size", file_size(entry.bytes)),
                    ],
                )
            })
            .collect::<Result<Vec<_>>>()?
            .join("\n");

        render(&self.index, &[("cards", cards)])
    }

    pub(super) fn viewer(&self, year: u16) -> Result<String> {
        render(&self.viewer, &[("year", year.to_string())])
    }
}

fn read(root: &Path, relative: &str) -> Result<String> {
    let path = root.join(relative);
    fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))
}

fn render(template: &str, values: &[(&str, String)]) -> Result<String> {
    let mut output = template.to_owned();
    for (name, value) in values {
        let placeholder = format!("{{{{{name}}}}}");
        if !output.contains(&placeholder) {
            bail!("site template is missing placeholder {placeholder}");
        }
        output = output.replace(&placeholder, value);
    }

    if let Some(start) = output.find("{{") {
        let remainder = &output[start..];
        let end = remainder.find("}}").map_or(remainder.len(), |end| end + 2);
        bail!(
            "site template contains unresolved placeholder {}",
            &remainder[..end]
        );
    }

    Ok(output)
}

fn file_size(bytes: u64) -> String {
    const MEBIBYTE: f64 = 1024.0 * 1024.0;
    format!("{:.1} MiB", bytes as f64 / MEBIBYTE)
}

#[cfg(test)]
mod tests {
    use super::{file_size, render};

    #[test]
    fn formats_file_size_in_mebibytes() {
        assert_eq!(file_size(1_572_864), "1.5 MiB");
    }

    #[test]
    fn rejects_missing_and_unresolved_placeholders() {
        assert!(render("plain", &[("year", "2026".to_owned())]).is_err());
        assert!(render("{{unknown}}", &[]).is_err());
    }
}
