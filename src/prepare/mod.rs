pub(crate) mod cards;
mod download;
mod images;
mod resources;

use anyhow::{Context, Result};

use crate::{limits, upstream::Upstream};

use self::download::Downloader;

pub(crate) async fn run(years: Vec<u16>) -> Result<()> {
    let project_root = std::env::current_dir().context("failed to resolve project root")?;
    let upstream = Upstream::at(&project_root);
    let mut limit_lists = limits::load_years(&years)?;
    let downloader = Downloader::new()?;
    resources::prepare(&downloader, &upstream).await?;
    let cards = cards::CardDatabase::load(&upstream.card_data())?;

    for limits in &mut limit_lists {
        limits.sort(&cards)?;
    }
    images::fetch(&downloader, &upstream.center_images(), &cards, &limit_lists).await?;
    for limits in &limit_lists {
        limits.write()?;
    }

    println!("sorted limit lists for {} year(s)", limit_lists.len());
    Ok(())
}
