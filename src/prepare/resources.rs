use anyhow::Result;

use crate::upstream::Upstream;

use super::download::Downloader;

const OT_CARDS_URL: &str = "https://github.com/arshtyi/ygo-cards/releases/download/latest/ot.json";

pub(super) async fn prepare(downloader: &Downloader, upstream: &Upstream) -> Result<()> {
    upstream.prepare_layout()?;
    downloader
        .download(OT_CARDS_URL, &upstream.card_data())
        .await?;
    upstream.validate_ready()?;

    println!(
        "prepared versioned template and assets at {}",
        upstream.template().display()
    );
    Ok(())
}
