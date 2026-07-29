use super::SiteYear;

pub(super) const CSS: &str = r#":root {
  color-scheme: dark;
  font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  background: #07090f;
  color: #f5f2e9;
  font-synthesis: none;
}

* {
  box-sizing: border-box;
}

body {
  min-width: 320px;
  margin: 0;
  background:
    radial-gradient(circle at 15% 0%, rgba(36, 93, 154, 0.24), transparent 38rem),
    radial-gradient(circle at 88% 18%, rgba(159, 101, 32, 0.18), transparent 32rem),
    #07090f;
}

a {
  color: inherit;
}

.shell {
  width: min(1120px, calc(100% - 40px));
  margin: 0 auto;
}

.hero {
  padding: 88px 0 52px;
}

.eyebrow {
  margin: 0 0 14px;
  color: #d5ad68;
  font-size: 0.75rem;
  font-weight: 800;
  letter-spacing: 0.18em;
  text-transform: uppercase;
}

h1,
h2,
p {
  margin-top: 0;
}

h1 {
  max-width: 850px;
  margin-bottom: 18px;
  font-family: Georgia, "Times New Roman", serif;
  font-size: clamp(2.6rem, 8vw, 6.5rem);
  font-weight: 500;
  letter-spacing: -0.055em;
  line-height: 0.92;
}

.intro {
  max-width: 620px;
  color: #aeb5c3;
  font-size: clamp(1rem, 2vw, 1.2rem);
  line-height: 1.7;
}

.archive {
  display: grid;
  gap: 18px;
  padding: 20px 0 84px;
}

.year-card {
  display: grid;
  grid-template-columns: minmax(120px, 0.42fr) minmax(240px, 1fr) auto;
  align-items: center;
  gap: 28px;
  padding: 30px;
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 20px;
  background: rgba(15, 18, 27, 0.82);
  box-shadow: 0 22px 60px rgba(0, 0, 0, 0.2);
}

.year {
  color: #d5ad68;
  font-family: Georgia, "Times New Roman", serif;
  font-size: clamp(2.7rem, 6vw, 5.2rem);
  line-height: 1;
}

.year-card h2 {
  margin-bottom: 8px;
  font-size: clamp(1.25rem, 2.5vw, 1.75rem);
}

.meta {
  margin-bottom: 0;
  color: #9199a8;
  font-size: 0.9rem;
}

.actions {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  justify-content: flex-end;
}

.button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-height: 44px;
  padding: 0 18px;
  border: 1px solid rgba(255, 255, 255, 0.16);
  border-radius: 999px;
  text-decoration: none;
  transition: border-color 150ms ease, background 150ms ease, transform 150ms ease;
}

.button:hover {
  border-color: rgba(213, 173, 104, 0.72);
  background: rgba(213, 173, 104, 0.1);
  transform: translateY(-1px);
}

.button.primary {
  border-color: #d5ad68;
  background: #d5ad68;
  color: #11131a;
  font-weight: 750;
}

.viewer-body {
  height: 100vh;
  overflow: hidden;
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
  gap: 24px;
  padding: 14px 20px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.12);
  background: #0d1018;
}

.viewer-title {
  min-width: 0;
}

.viewer-title h1 {
  overflow: hidden;
  margin: 0;
  font-family: inherit;
  font-size: clamp(1rem, 2vw, 1.35rem);
  font-weight: 700;
  letter-spacing: 0;
  line-height: 1.2;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.back {
  color: #aeb5c3;
  font-size: 0.82rem;
  text-decoration: none;
}

.document {
  display: block;
  width: 100%;
  height: 100%;
  border: 0;
  background: #252832;
}

.fallback {
  padding: 40px;
  color: #252832;
}

@media (max-width: 760px) {
  .shell {
    width: min(100% - 24px, 1120px);
  }

  .hero {
    padding-top: 56px;
  }

  .year-card {
    grid-template-columns: 1fr;
    gap: 18px;
    padding: 24px;
  }

  .actions {
    justify-content: flex-start;
  }

  .viewer-header {
    padding: 10px 12px;
  }

  .viewer-header .button {
    min-height: 40px;
    padding-inline: 14px;
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
        <div class="year">{year}</div>
        <div>
          <h2>{year} World Championship</h2>
          <p class="meta">OCG / TCG · PDF · {size}</p>
        </div>
        <div class="actions">
          <a class="button primary" href="{year}/">在线查看</a>
          <a class="button" href="{year}/lf.pdf" download>下载 PDF</a>
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
  <meta name="description" content="游戏王世界锦标赛 OCG / TCG 禁限卡表归档">
  <title>Yu-Gi-Oh! World Championship Limit Lists</title>
  <link rel="stylesheet" href="assets/site.css">
</head>
<body>
  <header class="hero shell">
    <p class="eyebrow">Official tournament archive</p>
    <h1>World Championship Limit Lists</h1>
    <p class="intro">游戏王世界锦标赛 OCG / TCG 禁止、限制与准限制卡表。选择年份在线查看，或下载完整 PDF。</p>
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
  <title>{year} World Championship Limit List</title>
  <link rel="stylesheet" href="../assets/site.css">
</head>
<body class="viewer-body">
  <main class="viewer-shell">
    <header class="viewer-header">
      <div class="viewer-title">
        <a class="back" href="../">← 返回全部年份</a>
        <h1>{year} World Championship Limit List</h1>
      </div>
      <a class="button primary" href="./lf.pdf" download>下载 PDF · {size}</a>
    </header>
    <object class="document" data="./lf.pdf" type="application/pdf">
      <p class="fallback">当前浏览器无法内嵌显示 PDF。<a href="./lf.pdf">打开或下载文件</a>。</p>
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
