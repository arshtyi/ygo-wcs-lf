use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

use crate::prepare::cards::CardDatabase;

type Groups = [[Vec<u32>; 3]; 2];

pub(crate) struct YearLimits {
    year: u16,
    path: PathBuf,
    groups: Groups,
}

impl YearLimits {
    pub(crate) fn sort(&mut self, cards: &CardDatabase) -> Result<()> {
        for group in self.groups.iter_mut().flatten() {
            cards
                .sort_ids(group)
                .with_context(|| format!("failed to sort {} limit list", self.year))?;
        }
        Ok(())
    }

    pub(crate) fn write(&self) -> Result<()> {
        let mut json =
            serde_json::to_vec_pretty(&self.groups).context("failed to encode sorted limits")?;
        json.push(b'\n');
        fs::write(&self.path, json)
            .with_context(|| format!("failed to write {}", self.path.display()))
    }

    pub(crate) fn year(&self) -> u16 {
        self.year
    }

    pub(crate) fn ids(&self) -> impl Iterator<Item = u32> + '_ {
        self.groups.iter().flatten().flatten().copied()
    }
}

pub(crate) fn load_years(years: &[u16]) -> Result<Vec<YearLimits>> {
    years
        .iter()
        .copied()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(load_year)
        .collect()
}

fn load_year(year: u16) -> Result<YearLimits> {
    let path = PathBuf::from(year.to_string()).join("data/lf.json");
    load_path(year, &path)
}

fn load_path(year: u16, path: &Path) -> Result<YearLimits> {
    let json = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    let groups = serde_json::from_slice(&json)
        .with_context(|| format!("invalid limit list structure in {}", path.display()))?;

    Ok(YearLimits {
        year,
        path: path.to_owned(),
        groups,
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::load_path;
    use crate::prepare::cards::CardDatabase;

    #[test]
    fn sorts_and_writes_all_six_groups() {
        let temp = tempdir().unwrap();
        let limits_path = temp.path().join("lf.json");
        let cards_path = temp.path().join("ot.json");
        fs::write(&limits_path, "[[[3,1,2],[],[]],[[4,3,2,1],[],[]]]").unwrap();
        fs::write(
            &cards_path,
            r#"[
                {"id":1,"image":101,"type":["怪兽","A","Z"]},
                {"id":2,"image":102,"type":["怪兽","Z","A"]},
                {"id":3,"image":103,"type":["魔法","通常"]},
                {"id":4,"image":104,"type":["陷阱","通常"]}
            ]"#,
        )
        .unwrap();
        let database = CardDatabase::load(&cards_path).unwrap();
        let mut limits = load_path(2026, &limits_path).unwrap();

        limits.sort(&database).unwrap();
        limits.write().unwrap();

        let actual: serde_json::Value =
            serde_json::from_slice(&fs::read(limits_path).unwrap()).unwrap();
        assert_eq!(
            actual,
            serde_json::json!([[[2, 1, 3], [], []], [[2, 1, 3, 4], [], []],])
        );
    }

    #[test]
    fn exposes_all_ids() {
        let temp = tempdir().unwrap();
        let path = temp.path().join("lf.json");
        fs::write(&path, "[[[1],[2],[3]],[[4],[5],[6]]]").unwrap();
        let limits = load_path(2026, &path).unwrap();

        assert_eq!(limits.ids().collect::<Vec<_>>(), [1, 2, 3, 4, 5, 6]);
    }
}
