use super::SiteYear;

pub(super) const CSS: &str = r#":root {
  font-family: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  background: #f5f5f5;
  color: #222;
}

* {
  box-sizing: border-box;
}

body {
  margin: 0;
  background: #f5f5f5;
}

a {
  color: inherit;
}

.shell {
  width: min(760px, calc(100% - 32px));
  margin: 0 auto;
}

.hero {
  padding: 48px 0 20px;
}

h1,
h2,
p {
  margin: 0;
}

h1 {
  margin-bottom: 6px;
  font-size: 1.8rem;
  line-height: 1.3;
}

.archive {
  display: grid;
  gap: 12px;
  padding: 12px 0 48px;
}

.year-card {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 20px;
  padding: 18px;
  border: 1px solid #ddd;
  border-radius: 8px;
  background: #fff;
}

.year-card h2 {
  margin-bottom: 4px;
  font-size: 1.3rem;
}

.meta {
  color: #666;
  font-size: 0.9rem;
}

.actions {
  display: flex;
  gap: 8px;
}

.button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-height: 38px;
  padding: 0 14px;
  border: 1px solid #bbb;
  border-radius: 6px;
  background: #fff;
  text-decoration: none;
}

.button:hover {
  background: #f0f0f0;
}

.button.primary {
  border-color: #222;
  background: #222;
  color: #fff;
}

.viewer-body {
  height: 100vh;
  overflow: hidden;
  background: #ddd;
}

.viewer-shell {
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  height: 100%;
}

.viewer-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 12px 16px;
  border-bottom: 1px solid #ddd;
  background: #fff;
}

.viewer-title {
  min-width: 0;
}

.viewer-title h1 {
  overflow: hidden;
  margin: 0;
  font-size: 1.1rem;
  line-height: 1.3;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.back {
  color: #666;
  font-size: 0.8rem;
}

.document {
  display: block;
  width: 100%;
  height: 100%;
  border: 0;
  background: #ddd;
}

.fallback {
  padding: 24px;
}

@media (max-width: 560px) {
  .hero {
    padding-top: 32px;
  }

  .year-card {
    align-items: flex-start;
    flex-direction: column;
  }

  .viewer-header {
    padding: 10px;
  }

  .viewer-header .button {
    flex: none;
  }
}
"#;

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
          <a class="button primary" href="{year}/">查看</a>
          <a class="button" href="{year}/lf.pdf" download>下载</a>
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

pub(super) fn viewer(year: u16, bytes: u64) -> String {
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
  <main class="viewer-shell">
    <header class="viewer-header">
      <div class="viewer-title">
        <a class="back" href="../">全部年份</a>
        <h1>{year} 世界赛禁限卡表</h1>
      </div>
      <a class="button primary" href="./lf.pdf" download>下载 · {size}</a>
    </header>
    <object class="document" data="./lf.pdf" type="application/pdf">
      <p class="fallback">无法显示 PDF。<a href="./lf.pdf">打开文件</a></p>
    </object>
  </main>
</body>
</html>
"#,
        size = file_size(bytes),
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
