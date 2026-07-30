use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{Context, Result};
use serde::Deserialize;
use tokio::sync::{Mutex, RwLock};

use crate::cache;

const SEARCH_ENDPOINT: &str = "https://ygocdb.com/api/v0/";

pub(super) trait CardSearch {
    async fn search_id(&self, name: &str) -> Result<Option<u32>>;
}

pub(super) struct CachedSearch<S> {
    source: S,
    path: PathBuf,
    entries: RwLock<BTreeMap<String, Option<u32>>>,
    save: Mutex<()>,
}

impl<S> CachedSearch<S> {
    pub(super) fn load(source: S, path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_owned();
        let entries = cache::read_json(&path)?.unwrap_or_default();

        Ok(Self {
            source,
            path,
            entries: RwLock::new(entries),
            save: Mutex::new(()),
        })
    }

    async fn cached(&self, name: &str) -> Option<Option<u32>> {
        self.entries.read().await.get(name).copied()
    }

    async fn store(&self, name: &str, result: Option<u32>) -> Result<Option<u32>> {
        let result = *self
            .entries
            .write()
            .await
            .entry(name.to_owned())
            .or_insert(result);
        let _save = self.save.lock().await;
        let entries = self.entries.read().await.clone();
        cache::write_json(&self.path, &entries)?;

        Ok(result)
    }
}

impl<S> CardSearch for CachedSearch<S>
where
    S: CardSearch,
{
    async fn search_id(&self, name: &str) -> Result<Option<u32>> {
        if let Some(result) = self.cached(name).await {
            return Ok(result);
        }

        let result = self.source.search_id(name).await?;
        self.store(name, result).await
    }
}

pub(super) struct ApiClient {
    client: reqwest::Client,
}

impl ApiClient {
    pub(super) fn new() -> Result<Self> {
        let client = reqwest::Client::builder()
            .user_agent(concat!(
                env!("CARGO_PKG_NAME"),
                "/",
                env!("CARGO_PKG_VERSION")
            ))
            .timeout(Duration::from_secs(20))
            .build()
            .context("failed to build HTTP client")?;

        Ok(Self { client })
    }
}

impl CardSearch for ApiClient {
    async fn search_id(&self, name: &str) -> Result<Option<u32>> {
        let response = self
            .client
            .get(SEARCH_ENDPOINT)
            .query(&[("search", name)])
            .send()
            .await
            .context("request failed")?
            .error_for_status()
            .context("API returned an error status")?
            .json::<SearchResponse>()
            .await
            .context("failed to decode API response")?;

        Ok(response.result.first().map(|card| card.id))
    }
}

#[derive(Deserialize)]
struct SearchResponse {
    result: Vec<SearchResult>,
}

#[derive(Deserialize)]
struct SearchResult {
    id: u32,
}

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeMap,
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        },
    };

    use anyhow::Result;
    use tempfile::tempdir;

    use super::{CachedSearch, CardSearch, SearchResponse};

    struct FakeSearch {
        answers: BTreeMap<String, Option<u32>>,
        calls: Arc<AtomicUsize>,
    }

    impl CardSearch for FakeSearch {
        async fn search_id(&self, name: &str) -> Result<Option<u32>> {
            self.calls.fetch_add(1, Ordering::Relaxed);
            Ok(*self.answers.get(name).expect("unexpected search"))
        }
    }

    #[test]
    fn reads_first_result_id() {
        let response: SearchResponse = serde_json::from_str(
            r#"{
                "result": [
                    {"id": 89631139, "jp_name": "青眼の白龍"},
                    {"id": 12345678}
                ]
            }"#,
        )
        .unwrap();

        assert_eq!(
            response.result.first().map(|card| card.id),
            Some(89_631_139)
        );
    }

    #[test]
    fn accepts_empty_results() {
        let response: SearchResponse = serde_json::from_str(r#"{"result":[]}"#).unwrap();

        assert!(response.result.is_empty());
    }

    #[tokio::test]
    async fn persists_found_and_missing_search_results() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("card-names.json");
        let calls = Arc::new(AtomicUsize::new(0));
        let search = CachedSearch::load(
            FakeSearch {
                answers: BTreeMap::from([
                    ("Blue-Eyes".to_owned(), Some(89_631_139)),
                    ("Missing".to_owned(), None),
                ]),
                calls: Arc::clone(&calls),
            },
            &path,
        )
        .unwrap();

        assert_eq!(
            search.search_id("Blue-Eyes").await.unwrap(),
            Some(89_631_139)
        );
        assert_eq!(search.search_id("Missing").await.unwrap(), None);
        assert_eq!(calls.load(Ordering::Relaxed), 2);
        drop(search);

        let reopened = CachedSearch::load(
            FakeSearch {
                answers: BTreeMap::new(),
                calls: Arc::clone(&calls),
            },
            &path,
        )
        .unwrap();

        assert_eq!(
            reopened.search_id("Blue-Eyes").await.unwrap(),
            Some(89_631_139)
        );
        assert_eq!(reopened.search_id("Missing").await.unwrap(), None);
        assert_eq!(calls.load(Ordering::Relaxed), 2);
    }
}
