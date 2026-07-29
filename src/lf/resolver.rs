use std::collections::{BTreeSet, HashMap};

use futures::{StreamExt, stream};

use super::{
    api::CardSearch,
    model::{CardEntry, Region, Restriction},
};

const MAX_CONCURRENT_REQUESTS: usize = 8;

#[derive(Debug, Eq, PartialEq)]
pub(super) struct ResolvedCard {
    pub(super) region: Region,
    pub(super) restriction: Restriction,
    pub(super) id: u32,
}

#[derive(Debug, Eq, PartialEq)]
pub(super) struct ResolveDiagnostic {
    pub(super) line_number: usize,
    pub(super) message: String,
}

#[derive(Debug)]
pub(super) struct ResolvedList {
    pub(super) cards: Vec<ResolvedCard>,
    pub(super) diagnostics: Vec<ResolveDiagnostic>,
}

pub(super) async fn resolve<S>(cards: &[CardEntry], search: &S) -> ResolvedList
where
    S: CardSearch,
{
    let names = cards
        .iter()
        .filter(|card| card.explicit_id.is_none())
        .flat_map(|card| {
            card.japanese_name
                .iter()
                .chain(card.english_name.iter())
                .cloned()
        })
        .collect::<BTreeSet<_>>();

    let search_results = stream::iter(names)
        .map(|name| async move {
            let result = search
                .search_id(&name)
                .await
                .map_err(|error| format!("{error:#}"));
            (name, result)
        })
        .buffer_unordered(MAX_CONCURRENT_REQUESTS)
        .collect::<HashMap<_, _>>()
        .await;

    let mut resolved = Vec::with_capacity(cards.len());
    let mut diagnostics = Vec::new();

    for card in cards {
        match resolve_card(card, &search_results) {
            Ok(id) => resolved.push(ResolvedCard {
                region: card.region,
                restriction: card.restriction,
                id,
            }),
            Err(message) => diagnostics.push(ResolveDiagnostic {
                line_number: card.line_number,
                message,
            }),
        }
    }

    ResolvedList {
        cards: resolved,
        diagnostics,
    }
}

fn resolve_card(
    card: &CardEntry,
    search_results: &HashMap<String, Result<Option<u32>, String>>,
) -> Result<u32, String> {
    if let Some(id) = card.explicit_id {
        return Ok(id);
    }

    match (&card.japanese_name, &card.english_name) {
        (Some(japanese_name), Some(english_name)) => {
            let japanese_id = resolved_name("Japanese", japanese_name, search_results)?;
            let english_id = resolved_name("English", english_name, search_results)?;

            if japanese_id != english_id {
                return Err(format!(
                    "name lookup mismatch: Japanese `{japanese_name}` returned {japanese_id}, \
                     English `{english_name}` returned {english_id}"
                ));
            }

            Ok(japanese_id)
        }
        (Some(japanese_name), None) => resolved_name("Japanese", japanese_name, search_results),
        (None, Some(english_name)) => resolved_name("English", english_name, search_results),
        (None, None) => Err("Japanese and English names are both empty".to_owned()),
    }
}

fn resolved_name(
    language: &str,
    name: &str,
    search_results: &HashMap<String, Result<Option<u32>, String>>,
) -> Result<u32, String> {
    match search_results.get(name) {
        Some(Ok(Some(id))) => Ok(*id),
        Some(Ok(None)) => Err(format!("no card found for {language} name `{name}`")),
        Some(Err(error)) => Err(format!(
            "failed to query {language} name `{name}`: {error}"
        )),
        None => Err(format!("missing query result for {language} name `{name}`")),
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };

    use anyhow::{Result, anyhow};

    use super::{ResolveDiagnostic, ResolvedCard, resolve};
    use crate::lf::{
        api::CardSearch,
        model::{CardEntry, Region, Restriction},
    };

    struct FakeSearch {
        answers: HashMap<String, Answer>,
        calls: Arc<Mutex<Vec<String>>>,
    }

    enum Answer {
        Found(u32),
        Missing,
        Failed,
    }

    impl CardSearch for FakeSearch {
        async fn search_id(&self, name: &str) -> Result<Option<u32>> {
            self.calls.lock().unwrap().push(name.to_owned());

            match self.answers.get(name).unwrap() {
                Answer::Found(id) => Ok(Some(*id)),
                Answer::Missing => Ok(None),
                Answer::Failed => Err(anyhow!("offline")),
            }
        }
    }

    #[tokio::test]
    async fn resolves_bilingual_and_single_name_cards() {
        let search = fake_search([
            ("青眼の白龍", Answer::Found(89_631_139)),
            ("Blue-Eyes White Dragon", Answer::Found(89_631_139)),
            ("増殖するＧ", Answer::Found(23_427_709)),
        ]);
        let cards = [
            card(
                Some("青眼の白龍"),
                Some("Blue-Eyes White Dragon"),
                None,
                3,
            ),
            card(Some("増殖するＧ"), None, None, 4),
        ];

        let resolved = resolve(&cards, &search).await;

        assert_eq!(
            resolved.cards,
            vec![
                ResolvedCard {
                    region: Region::Ocg,
                    restriction: Restriction::Forbidden,
                    id: 89_631_139,
                },
                ResolvedCard {
                    region: Region::Ocg,
                    restriction: Restriction::Forbidden,
                    id: 23_427_709,
                },
            ]
        );
        assert!(resolved.diagnostics.is_empty());
    }

    #[tokio::test]
    async fn explicit_ids_bypass_queries_and_names_are_cached() {
        let search = fake_search([("Shared", Answer::Found(7))]);
        let cards = [
            card(None, Some("Manual"), Some(42), 1),
            card(None, Some("Shared"), None, 2),
            card(None, Some("Shared"), None, 3),
        ];

        let resolved = resolve(&cards, &search).await;

        assert_eq!(
            resolved.cards.iter().map(|card| card.id).collect::<Vec<_>>(),
            [42, 7, 7]
        );
        assert_eq!(*search.calls.lock().unwrap(), ["Shared"]);
    }

    #[tokio::test]
    async fn skips_mismatches_missing_results_and_request_failures() {
        let search = fake_search([
            ("Japanese", Answer::Found(1)),
            ("English", Answer::Found(2)),
            ("Missing", Answer::Missing),
            ("Failed", Answer::Failed),
        ]);
        let cards = [
            card(Some("Japanese"), Some("English"), None, 10),
            card(None, Some("Missing"), None, 11),
            card(Some("Failed"), None, None, 12),
        ];

        let resolved = resolve(&cards, &search).await;

        assert!(resolved.cards.is_empty());
        assert_eq!(
            resolved.diagnostics,
            vec![
                ResolveDiagnostic {
                    line_number: 10,
                    message: "name lookup mismatch: Japanese `Japanese` returned 1, English `English` returned 2".to_owned(),
                },
                ResolveDiagnostic {
                    line_number: 11,
                    message: "no card found for English name `Missing`".to_owned(),
                },
                ResolveDiagnostic {
                    line_number: 12,
                    message: "failed to query Japanese name `Failed`: offline".to_owned(),
                },
            ]
        );
    }

    fn fake_search<const N: usize>(answers: [(&str, Answer); N]) -> FakeSearch {
        FakeSearch {
            answers: answers
                .into_iter()
                .map(|(name, answer)| (name.to_owned(), answer))
                .collect(),
            calls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn card(
        japanese_name: Option<&str>,
        english_name: Option<&str>,
        explicit_id: Option<u32>,
        line_number: usize,
    ) -> CardEntry {
        CardEntry {
            region: Region::Ocg,
            restriction: Restriction::Forbidden,
            japanese_name: japanese_name.map(str::to_owned),
            english_name: english_name.map(str::to_owned),
            explicit_id,
            line_number,
        }
    }
}
