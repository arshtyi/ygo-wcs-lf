mod pdf;
mod previews;
mod typst;

use anyhow::Result;

use crate::limits;

const PREVIEW_PPI: u16 = 72;

pub(crate) fn run(years: Vec<u16>) -> Result<()> {
    let limit_lists = limits::load_years(&years)?;
    previews::compile(&limit_lists, PREVIEW_PPI)?;
    pdf::compile(&limit_lists)?;
    Ok(())
}
