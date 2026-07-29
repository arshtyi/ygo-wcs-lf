#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Region {
    Ocg,
    Tcg,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Restriction {
    Forbidden,
    Limited,
    SemiLimited,
}

#[derive(Debug, Eq, PartialEq)]
pub(super) struct CardEntry {
    pub(super) region: Region,
    pub(super) restriction: Restriction,
    pub(super) japanese_name: Option<String>,
    pub(super) english_name: Option<String>,
    pub(super) explicit_id: Option<u32>,
    pub(super) line_number: usize,
}
