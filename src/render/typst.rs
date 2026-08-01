use std::{path::Path, process::Command};

use anyhow::{Result, bail};

use crate::upstream::Upstream;

const TYPST: &str = "typst";

pub(super) fn command(project_root: &Path) -> Result<Command> {
    let upstream = Upstream::at(project_root);
    upstream.validate_ready()?;
    let font_paths = upstream.font_directories();
    for path in &font_paths {
        if !path.is_dir() {
            bail!(
                "required Typst font directory is missing: {}; run `prepare` first",
                path.display()
            );
        }
    }

    let mut command = Command::new(TYPST);
    command.arg("compile").arg("--root").arg(project_root);
    for path in font_paths {
        command.arg("--font-path").arg(path);
    }
    Ok(command)
}
