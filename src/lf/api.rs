use std::time::Duration;

use anyhow::{Context, Result};
use serde::Deserialize;

const SEARCH_ENDPOINT: &str = "https://ygocdb.com/api/v0/";

pub(super) trait CardSearch {
    async fn search_id(&self, name: &str) -> Result<Option<u32>>;
}

pub(super) struct ApiClient {
    client: reqwest::Client,
}

impl ApiClient {
    pub(super) fn new() -> Result<Self> {
        let client = reqwest::Client::builder()
            .user_agent(concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")))
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
    use super::SearchResponse;

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

        assert_eq!(response.result.first().map(|card| card.id), Some(89_631_139));
    }

    #[test]
    fn accepts_empty_results() {
        let response: SearchResponse = serde_json::from_str(r#"{"result":[]}"#).unwrap();

        assert!(response.result.is_empty());
    }
}
