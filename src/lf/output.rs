use std::{fs, path::Path};

use anyhow::{Context, Result};

use super::{
    model::{Region, Restriction},
    resolver::ResolvedCard,
};

pub(super) fn write(path: &Path, cards: &[ResolvedCard]) -> Result<()> {
    let mut json = serialize(cards)?;
    json.push(b'\n');

    fs::write(path, json).with_context(|| format!("failed to write {}", path.display()))
}

fn serialize(cards: &[ResolvedCard]) -> Result<Vec<u8>> {
    let mut lists: [[Vec<u32>; 3]; 2] = Default::default();

    for card in cards {
        lists[region_index(card.region)][restriction_index(card.restriction)].push(card.id);
    }

    serde_json::to_vec_pretty(&lists).context("failed to encode limit list as JSON")
}

const fn region_index(region: Region) -> usize {
    match region {
        Region::Ocg => 0,
        Region::Tcg => 1,
    }
}

const fn restriction_index(restriction: Restriction) -> usize {
    match restriction {
        Restriction::Forbidden => 0,
        Restriction::Limited => 1,
        Restriction::SemiLimited => 2,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::serialize;
    use crate::lf::{
        model::{Region, Restriction},
        resolver::ResolvedCard,
    };

    #[test]
    fn serializes_regions_and_restrictions_in_fixed_order() {
        let cards = [
            resolved(Region::Tcg, Restriction::SemiLimited, 6),
            resolved(Region::Ocg, Restriction::Forbidden, 1),
            resolved(Region::Tcg, Restriction::Forbidden, 4),
            resolved(Region::Ocg, Restriction::Limited, 2),
            resolved(Region::Ocg, Restriction::SemiLimited, 3),
            resolved(Region::Tcg, Restriction::Limited, 5),
            resolved(Region::Ocg, Restriction::Forbidden, 7),
        ];

        let json: serde_json::Value = serde_json::from_slice(&serialize(&cards).unwrap()).unwrap();

        assert_eq!(json, json!([[[1, 7], [2], [3]], [[4], [5], [6]],]));
    }

    fn resolved(region: Region, restriction: Restriction, id: u32) -> ResolvedCard {
        ResolvedCard {
            region,
            restriction,
            id,
        }
    }
}
