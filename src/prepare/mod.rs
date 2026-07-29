mod archive;
mod cards;
mod download;
mod limits;
mod workspace;

use anyhow::Result;

use self::download::Downloader;

pub(crate) async fn run(years: Vec<u16>) -> Result<()> {
    let mut limit_lists = limits::load_years(&years)?;
    let downloader = Downloader::new()?;
    let workspace = workspace::assemble(&downloader).await?;
    let cards = cards::CardDatabase::load(&workspace.join("assets/ot/card/ot.json"))?;

    for limits in &mut limit_lists {
        limits.sort(&cards)?;
    }
    for limits in &limit_lists {
        limits.write()?;
    }

    println!("sorted limit lists for {} year(s)", limit_lists.len());
    Ok(())
}
