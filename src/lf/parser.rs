use super::model::{CardEntry, Region, Restriction};

#[derive(Debug, Eq, PartialEq)]
pub(super) struct Diagnostic {
    pub(super) line_number: usize,
    pub(super) message: String,
}

#[derive(Debug)]
pub(super) struct ParsedList {
    pub(super) cards: Vec<CardEntry>,
    pub(super) diagnostics: Vec<Diagnostic>,
}

#[derive(Default)]
struct Section {
    region: Option<Region>,
    restriction: Option<Restriction>,
}

pub(super) fn parse(source: &str) -> ParsedList {
    let mut section = Section::default();
    let mut cards = Vec::new();
    let mut diagnostics = Vec::new();

    for (index, raw_line) in source.lines().enumerate() {
        let line_number = index + 1;
        let line = raw_line.trim();

        if line.is_empty() {
            continue;
        }

        if let Some(marker) = line.strip_prefix("///") {
            parse_marker(marker.trim(), line_number, &mut section, &mut diagnostics);
            continue;
        }

        match parse_card(line, line_number, &section) {
            Ok(card) => cards.push(card),
            Err(message) => diagnostics.push(Diagnostic {
                line_number,
                message,
            }),
        }
    }

    ParsedList { cards, diagnostics }
}

fn parse_marker(
    marker: &str,
    line_number: usize,
    section: &mut Section,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if marker.starts_with("ocg") {
        section.region = Some(Region::Ocg);
        section.restriction = None;
        return;
    }

    if marker.starts_with("tcg") {
        section.region = Some(Region::Tcg);
        section.restriction = None;
        return;
    }

    let restriction = match marker {
        "forbidden" => Restriction::Forbidden,
        "limited" => Restriction::Limited,
        "semi-limited" => Restriction::SemiLimited,
        _ => {
            diagnostics.push(Diagnostic {
                line_number,
                message: format!("unknown section marker `{marker}`"),
            });
            return;
        }
    };

    if section.region.is_none() {
        diagnostics.push(Diagnostic {
            line_number,
            message: format!("restriction marker `{marker}` appears before a region"),
        });
        return;
    }

    section.restriction = Some(restriction);
}

fn parse_card(line: &str, line_number: usize, section: &Section) -> Result<CardEntry, String> {
    let region = section
        .region
        .ok_or_else(|| "card appears before an ocg or tcg marker".to_owned())?;
    let restriction = section
        .restriction
        .ok_or_else(|| "card appears before a restriction marker".to_owned())?;
    let fields: Vec<_> = line.split("//").map(str::trim).collect();

    if !(2..=3).contains(&fields.len()) {
        return Err(format!(
            "expected `Japanese name // English name[ // id]`, got {line:?}"
        ));
    }

    let japanese_name = non_empty(fields[0]);
    let english_name = non_empty(fields[1]);

    if japanese_name.is_none() && english_name.is_none() {
        return Err("Japanese and English names are both empty".to_owned());
    }

    let explicit_id = fields
        .get(2)
        .map(|value| {
            if value.is_empty() {
                return Err("explicit ID is empty".to_owned());
            }

            value
                .parse::<u32>()
                .map_err(|_| format!("invalid explicit ID `{value}`"))
        })
        .transpose()?;

    Ok(CardEntry {
        region,
        restriction,
        japanese_name,
        english_name,
        explicit_id,
        line_number,
    })
}

fn non_empty(value: &str) -> Option<String> {
    (!value.is_empty()).then(|| value.to_owned())
}

#[cfg(test)]
mod tests {
    use super::{Diagnostic, parse};
    use crate::lf::model::{CardEntry, Region, Restriction};

    #[test]
    fn parses_sections_names_and_explicit_ids() {
        let source = "\
/// ocg: source
/// forbidden
青眼の白龍 // Blue-Eyes White Dragon // 89631139
/// limited
増殖するＧ //
/// tcg: source
/// semi-limited
// Crossout Designator
";

        let parsed = parse(source);

        assert!(parsed.diagnostics.is_empty());
        assert_eq!(
            parsed.cards,
            vec![
                CardEntry {
                    region: Region::Ocg,
                    restriction: Restriction::Forbidden,
                    japanese_name: Some("青眼の白龍".to_owned()),
                    english_name: Some("Blue-Eyes White Dragon".to_owned()),
                    explicit_id: Some(89_631_139),
                    line_number: 3,
                },
                CardEntry {
                    region: Region::Ocg,
                    restriction: Restriction::Limited,
                    japanese_name: Some("増殖するＧ".to_owned()),
                    english_name: None,
                    explicit_id: None,
                    line_number: 5,
                },
                CardEntry {
                    region: Region::Tcg,
                    restriction: Restriction::SemiLimited,
                    japanese_name: None,
                    english_name: Some("Crossout Designator".to_owned()),
                    explicit_id: None,
                    line_number: 8,
                },
            ]
        );
    }

    #[test]
    fn skips_malformed_lines_with_diagnostics() {
        let source = "\
card before markers // Card
/// ocg
/// forbidden
missing delimiter
//
Japanese // English // nope
Japanese // English // 1 // extra
";

        let parsed = parse(source);

        assert!(parsed.cards.is_empty());
        assert_eq!(
            parsed.diagnostics,
            vec![
                Diagnostic {
                    line_number: 1,
                    message: "card appears before an ocg or tcg marker".to_owned(),
                },
                Diagnostic {
                    line_number: 4,
                    message:
                        "expected `Japanese name // English name[ // id]`, got \"missing delimiter\""
                            .to_owned(),
                },
                Diagnostic {
                    line_number: 5,
                    message: "Japanese and English names are both empty".to_owned(),
                },
                Diagnostic {
                    line_number: 6,
                    message: "invalid explicit ID `nope`".to_owned(),
                },
                Diagnostic {
                    line_number: 7,
                    message: "expected `Japanese name // English name[ // id]`, got \"Japanese // English // 1 // extra\"".to_owned(),
                },
            ]
        );
    }

    #[test]
    fn parses_current_project_lists_without_diagnostics() {
        let lists = [
            include_str!("../../2025/data/lf.list"),
            include_str!("../../2026/data/lf.list"),
        ];

        let parsed: Vec<_> = lists.into_iter().map(parse).collect();

        assert_eq!(parsed[0].cards.len(), 897);
        assert_eq!(parsed[1].cards.len(), 906);
        assert!(parsed.iter().all(|list| list.diagnostics.is_empty()));
    }
}
