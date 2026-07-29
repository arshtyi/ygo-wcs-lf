mod previews;

use anyhow::{Result, bail};

use crate::limits;

const PREVIEW_PPI: u16 = 72;

pub(crate) fn run(years: Vec<u16>) -> Result<()> {
    let limit_lists = limits::load_years(&years)?;
    previews::compile(&limit_lists, PREVIEW_PPI)?;

    bail!(
        "rendered card previews for {} year(s); PDF compilation is not implemented yet",
        limit_lists.len()
    )
}
