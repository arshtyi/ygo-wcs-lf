use super::SiteYear;

pub(super) fn index(years: &[SiteYear]) -> String {
    let cards = years
        .iter()
        .map(|entry| {
            let year = entry.year;
            format!(
                r#"      <article class="year-card">
        <div>
          <h2>{year}</h2>
          <p class="meta">OCG / TCG · {size}</p>
        </div>
        <div class="actions">
          <a class="button primary" href="{year}/index.html">查看</a>
        </div>
      </article>"#,
                size = file_size(entry.bytes),
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"<!doctype html>
<html lang="zh-CN">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <meta name="description" content="游戏王世界锦标赛 OCG / TCG 禁限卡表">
  <title>游戏王世界赛禁限卡表</title>
  <link rel="stylesheet" href="assets/site.css">
</head>
<body>
  <header class="hero shell">
    <h1>游戏王世界赛禁限卡表</h1>
    <p class="meta">OCG / TCG</p>
  </header>
  <main class="archive shell">
{cards}
  </main>
</body>
</html>
"#
    )
}

pub(super) fn viewer(year: u16) -> String {
    format!(
        r#"<!doctype html>
<html lang="zh-CN">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <meta name="description" content="{year} 游戏王世界锦标赛 OCG / TCG 禁限卡表">
  <title>{year} 世界赛禁限卡表</title>
  <link rel="stylesheet" href="../assets/site.css">
</head>
<body class="viewer-body">
  <iframe class="document" src="./lf.pdf#view=FitH" title="{year} 世界赛禁限卡表"></iframe>
</body>
</html>
"#
    )
}

fn file_size(bytes: u64) -> String {
    const MEBIBYTE: f64 = 1024.0 * 1024.0;
    format!("{:.1} MiB", bytes as f64 / MEBIBYTE)
}

#[cfg(test)]
mod tests {
    use super::file_size;

    #[test]
    fn formats_file_size_in_mebibytes() {
        assert_eq!(file_size(1_572_864), "1.5 MiB");
    }
}
