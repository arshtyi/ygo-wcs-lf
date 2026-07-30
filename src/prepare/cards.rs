use std::{collections::HashMap, fs, path::Path};

use anyhow::{Context, Result, bail};
use serde::Deserialize;

const MONSTER: &str = "怪兽";
const SPELL: &str = "魔法";
const TRAP: &str = "陷阱";

#[derive(Deserialize)]
struct Card {
    id: u32,
    image: Option<u32>,
    #[serde(rename = "type")]
    card_type: Option<Vec<String>>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct SortKey {
    kind: u8,
    details: Vec<String>,
}

pub(crate) struct CardDatabase {
    cards: HashMap<u32, Card>,
}

impl CardDatabase {
    pub(crate) fn load(path: &Path) -> Result<Self> {
        let json = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
        Self::from_json(&json).with_context(|| format!("invalid card data in {}", path.display()))
    }

    pub(crate) fn sort_ids(&self, ids: &mut [u32]) -> Result<()> {
        let keys = ids
            .iter()
            .map(|id| Ok((*id, self.sort_key(*id)?)))
            .collect::<Result<HashMap<_, _>>>()?;

        ids.sort_by(|left, right| keys[left].cmp(&keys[right]));
        Ok(())
    }

    pub(crate) fn image_id(&self, id: u32) -> Result<u32> {
        self.card(id)?
            .image
            .ok_or_else(|| anyhow::anyhow!("card {id} has no image ID"))
    }

    fn from_json(json: &[u8]) -> Result<Self> {
        let cards =
            serde_json::from_slice::<Vec<Card>>(json).context("expected a JSON array of cards")?;
        let mut index = HashMap::with_capacity(cards.len());

        for card in cards {
            let id = card.id;
            if index.insert(id, card).is_some() {
                bail!("duplicate card ID {id}");
            }
        }

        Ok(Self { cards: index })
    }

    fn card(&self, id: u32) -> Result<&Card> {
        self.cards
            .get(&id)
            .ok_or_else(|| anyhow::anyhow!("card {id} is missing from OT card data"))
    }

    fn sort_key(&self, id: u32) -> Result<SortKey> {
        let card_type = self
            .card(id)?
            .card_type
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("card {id} has no type"))?;

        if card_type.is_empty() || card_type.iter().any(String::is_empty) {
            bail!("card {id} has an invalid type: {card_type:?}");
        }

        let kind = match card_type[0].as_str() {
            MONSTER => 0,
            SPELL => 1,
            TRAP => 2,
            _ => 3,
        };
        let details = if card_type[0] == MONSTER {
            if card_type.len() < 3 {
                bail!("monster card {id} has an invalid type: {card_type:?}");
            }

            card_type[2..]
                .iter()
                .chain(&card_type[1..2])
                .cloned()
                .collect()
        } else {
            card_type[1..].to_vec()
        };

        Ok(SortKey { kind, details })
    }
}

#[cfg(test)]
mod tests {
    use super::CardDatabase;

    #[test]
    fn sorts_kinds_then_type_with_monster_race_last() {
        let database = cards(
            r#"[
                {"id":1,"image":101,"type":["怪兽","A-race","Z-category"]},
                {"id":2,"image":102,"type":["怪兽","Z-race","A-category"]},
                {"id":3,"image":103,"type":["魔法","速攻"]},
                {"id":4,"image":104,"type":["陷阱","反击"]},
                {"id":5,"image":105,"type":["怪兽","A-race","A-category"]}
            ]"#,
        );
        let mut ids = [4, 1, 3, 2, 5];

        database.sort_ids(&mut ids).unwrap();

        assert_eq!(ids, [5, 2, 1, 3, 4]);
    }

    #[test]
    fn keeps_source_order_for_equal_type_keys() {
        let database = cards(
            r#"[
                {"id":1,"image":101,"type":["魔法","通常"]},
                {"id":2,"image":102,"type":["魔法","通常"]}
            ]"#,
        );
        let mut ids = [2, 1];

        database.sort_ids(&mut ids).unwrap();

        assert_eq!(ids, [2, 1]);
    }

    #[test]
    fn rejects_missing_cards_and_invalid_monster_types() {
        let database = cards(r#"[{"id":1,"image":101,"type":["怪兽","龙族"]}]"#);

        assert!(database.sort_ids(&mut [1]).is_err());
        assert!(database.sort_ids(&mut [2]).is_err());
    }

    #[test]
    fn resolves_center_image_ids() {
        let database = cards(r#"[{"id":1,"image":101,"type":["魔法","通常"]}]"#);

        assert_eq!(database.image_id(1).unwrap(), 101);
    }

    fn cards(json: &str) -> CardDatabase {
        CardDatabase::from_json(json.as_bytes()).unwrap()
    }
}
