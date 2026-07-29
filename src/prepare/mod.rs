mod archive;
mod download;
mod workspace;

use anyhow::{Result, bail};

use self::download::Downloader;

pub(crate) async fn run(years: Vec<u16>) -> Result<()> {
    let downloader = Downloader::new()?;
    workspace::assemble(&downloader).await?;

    bail!(
        "prepared upstream workspace for {} year(s); sorting is not implemented yet",
        years.len()
    )
}
