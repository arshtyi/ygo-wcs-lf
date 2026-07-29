#let page-width = 410mm
#let page-margin = 12mm
#let cards-per-row = 10
#let card-gap = 0mm
#let content-width = page-width - 2 * page-margin
#let card-width = (content-width - (cards-per-row - 1) * card-gap) / cards-per-row
#let card-height = card-width * 2031 / 1394
#let background = rgb("#f3f4f6")
#let ink = rgb("#172033")
#let surface-line = rgb("#d7dce5")

#let row-count(count) = calc.ceil(count / cards-per-row)

#let market-height(groups) = (
  87mm
    + groups.fold(
      0mm,
      (height, ids) => height + 18mm + row-count(ids.len()) * (card-height + card-gap),
    )
)

#let preview-path(id) = "/build/previews/ot/" + str(id) + ".png"

#let statistic(label, value, color) = block(
  fill: color.lighten(88%),
  stroke: 0.6pt + color.lighten(62%),
  radius: 2mm,
  inset: (x: 4mm, y: 3mm),
  [
    #text(size: 15pt, weight: "bold", fill: color, smallcaps(label)) \
    #text(size: 18pt, weight: "bold", str(value))
  ],
)

#let restriction(title, ids, color) = {
  block(
    fill: color.lighten(92%),
    stroke: 0.6pt + color.lighten(70%),
    radius: 2mm,
    inset: (x: 4mm, y: 2.5mm),
    width: 100%,
    grid(
      columns: (1.2mm, 1fr, auto),
      column-gutter: 3mm,
      align: (center + horizon, left + horizon, right + horizon),
      rect(width: 100%, height: 5mm, radius: 0.6mm, fill: color),
      text(size: 18pt, weight: "bold", fill: color, smallcaps(title)),
      text(size: 16pt, weight: "bold", fill: color.darken(12%))[
        #ids.len() #smallcaps[Cards]
      ],
    ),
  )
  v(2mm)
  grid(
    columns: cards-per-row,
    column-gutter: card-gap,
    row-gutter: card-gap,
    ..ids.map(id => image(preview-path(id), width: card-width)),
  )
}

#let market(title, groups, accent) = {
  let total = groups.map(ids => ids.len()).sum()
  block(
    fill: white,
    stroke: 0.7pt + surface-line,
    radius: 3mm,
    inset: 6mm,
    width: 100%,
    grid(
      columns: (auto, 1fr),
      column-gutter: 12mm,
      align: (left + horizon, right + horizon),
      text(size: 45pt, weight: "bold", fill: accent, smallcaps(title)),
      grid(
        columns: 4,
        gutter: 3mm,
        statistic("Forbidden", groups.at(0).len(), rgb("#c0392b")),
        statistic("Limited", groups.at(1).len(), rgb("#d97706")),
        statistic("Semi-Limited", groups.at(2).len(), rgb("#2563eb")),
        statistic("Total", total, accent),
      ),
    ),
  )
  v(7mm)
  restriction("Forbidden", groups.at(0), rgb("#c0392b"))
  v(7mm)
  restriction("Limited", groups.at(1), rgb("#d97706"))
  v(7mm)
  restriction("Semi-Limited", groups.at(2), rgb("#2563eb"))
}

#let render-limit-list(year, limits) = {
  set text(fill: ink)
  set par(leading: 0.65em)
  set page(width: page-width, margin: page-margin, fill: background)

  set page(height: market-height(limits.at(0)))
  market(str(year) + " WCS OCG", limits.at(0), rgb("#7c3aed"))
  pagebreak()
  set page(height: market-height(limits.at(1)))
  market(str(year) + " WCS TCG", limits.at(1), rgb("#0f766e"))
}
