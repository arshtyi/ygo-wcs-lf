use super::SiteYear;

pub(super) const CSS: &str = r#":root {
  color-scheme: dark;
  font-family: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  background: #0d1218;
  color: #dce5ee;
}

* {
  box-sizing: border-box;
}

body {
  margin: 0;
  background: #0d1218;
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
  border: 1px solid #273442;
  border-radius: 8px;
  background: #141c25;
}

.year-card h2 {
  margin-bottom: 4px;
  font-size: 1.3rem;
}

.meta {
  color: #91a4b7;
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
  border: 1px solid #40556a;
  border-radius: 6px;
  background: #141c25;
  text-decoration: none;
}

.button:hover {
  background: #1b2733;
}

.button.primary {
  border-color: #3579a8;
  background: #3579a8;
  color: #fff;
}

.button[aria-busy="true"] {
  cursor: wait;
  opacity: 0.7;
  pointer-events: none;
}

.viewer-body {
  height: 100vh;
  overflow: hidden;
  background: #090d12;
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
  border-bottom: 1px solid #273442;
  background: #111821;
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
  color: #8eb8d6;
  font-size: 0.8rem;
}

.document {
  display: block;
  width: 100%;
  height: 100%;
  border: 0;
  background: #090d12;
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

pub(super) const DOWNLOAD_JS: &str = r#"for (const link of document.querySelectorAll("[data-download]")) {
  link.addEventListener("click", async (event) => {
    event.preventDefault();
    const label = link.textContent;
    link.setAttribute("aria-busy", "true");
    link.textContent = "下载中";

    try {
      const response = await fetch(link.href);
      if (!response.ok) throw new Error(response.statusText);
      const url = URL.createObjectURL(await response.blob());
      const download = document.createElement("a");
      download.href = url;
      download.download = link.download;
      document.body.append(download);
      download.click();
      download.remove();
      setTimeout(() => URL.revokeObjectURL(url), 1000);
    } catch {
      const download = document.createElement("a");
      download.href = link.href;
      download.download = link.download;
      document.body.append(download);
      download.click();
      download.remove();
    } finally {
      link.removeAttribute("aria-busy");
      link.textContent = label;
    }
  });
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
          <a class="button primary" href="{year}/index.html">查看</a>
          <a class="button" href="{year}/lf.pdf" download="{year}-world-championship-limit-list.pdf" data-download>下载</a>
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
  <script defer src="assets/site.js"></script>
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
  <script defer src="../assets/site.js"></script>
</head>
<body class="viewer-body">
  <main class="viewer-shell">
    <header class="viewer-header">
      <div class="viewer-title">
        <a class="back" href="../">全部年份</a>
        <h1>{year} 世界赛禁限卡表</h1>
      </div>
      <a class="button primary" href="./lf.pdf" download="{year}-world-championship-limit-list.pdf" data-download>下载 · {size}</a>
    </header>
    <iframe class="document" src="./lf.pdf#view=FitH" title="{year} 世界赛禁限卡表"></iframe>
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
