use std::{path::Path, time::Duration};

use anyhow::{Context, Result, anyhow};
use futures::StreamExt;
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
    time::sleep,
};

const ATTEMPTS: u8 = 3;

pub(super) struct Downloader {
    client: reqwest::Client,
}

impl Downloader {
    pub(super) fn new() -> Result<Self> {
        let client = reqwest::Client::builder()
            .user_agent(concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")))
            .timeout(Duration::from_secs(120))
            .build()
            .context("failed to build download client")?;

        Ok(Self { client })
    }

    pub(super) async fn download(&self, url: &str, destination: &Path) -> Result<()> {
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)
                .await
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }

        let mut last_error = None;

        for attempt in 1..=ATTEMPTS {
            match self.download_once(url, destination).await {
                Ok(()) => return Ok(()),
                Err(error) => {
                    last_error = Some(error);
                    let _ = fs::remove_file(destination).await;
                    if attempt < ATTEMPTS {
                        sleep(Duration::from_secs(u64::from(attempt))).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow!("download failed")))
            .with_context(|| format!("failed to download {url}"))
    }

    async fn download_once(&self, url: &str, destination: &Path) -> Result<()> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .context("request failed")?
            .error_for_status()
            .context("server returned an error status")?;
        let mut chunks = response.bytes_stream();
        let mut output = File::create(destination)
            .await
            .with_context(|| format!("failed to create {}", destination.display()))?;

        while let Some(chunk) = chunks.next().await {
            output
                .write_all(&chunk.context("failed to read response body")?)
                .await
                .with_context(|| format!("failed to write {}", destination.display()))?;
        }

        output
            .flush()
            .await
            .with_context(|| format!("failed to flush {}", destination.display()))
    }
}
