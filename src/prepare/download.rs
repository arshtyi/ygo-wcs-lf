use std::{path::Path, time::Duration};

use anyhow::{Context, Result, anyhow};
use futures::StreamExt;
use tempfile::NamedTempFile;
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
    time::sleep,
};

const ATTEMPTS: u8 = 3;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);
const READ_TIMEOUT: Duration = Duration::from_secs(120);

pub(super) struct Downloader {
    client: reqwest::Client,
}

impl Downloader {
    pub(super) fn new() -> Result<Self> {
        let client = reqwest::Client::builder()
            .user_agent(concat!(
                env!("CARGO_PKG_NAME"),
                "/",
                env!("CARGO_PKG_VERSION")
            ))
            .connect_timeout(CONNECT_TIMEOUT)
            .read_timeout(READ_TIMEOUT)
            .build()
            .context("failed to build download client")?;

        Ok(Self { client })
    }

    pub(super) async fn download(&self, url: &str, destination: &Path) -> Result<()> {
        self.download_checked(url, destination, |_| Ok(())).await
    }

    pub(super) async fn download_checked<F>(
        &self,
        url: &str,
        destination: &Path,
        validate: F,
    ) -> Result<()>
    where
        F: Fn(&Path) -> Result<()>,
    {
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)
                .await
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        let parent = destination
            .parent()
            .context("download destination has no parent")?;

        let mut last_error = None;

        for attempt in 1..=ATTEMPTS {
            let temporary = NamedTempFile::new_in(parent)
                .context("failed to create temporary download")?
                .into_temp_path();
            let result = match self.download_once(url, &temporary).await {
                Ok(()) => validate(&temporary),
                Err(error) => Err(error),
            };

            match result {
                Ok(()) => {
                    temporary
                        .persist(destination)
                        .with_context(|| format!("failed to replace {}", destination.display()))?;
                    return Ok(());
                }
                Err(error) => {
                    last_error = Some(error);
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
