use std::fs;
use std::process::Command;

use neospeller::firestore_logger::{fetch_all, SpellcheckLog};
use similar::{ChangeTag, TextDiff};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("Fetching logs from Firestore...");
    let mut logs = fetch_all().await?;
    logs.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    eprintln!("Got {} logs. Writing logs.html...", logs.len());
    let html = render(&logs);
    fs::write("logs.html", html)?;

    eprintln!("Opening...");
    let _ = Command::new("open").arg("logs.html").status();

    Ok(())
}

fn render(logs: &[SpellcheckLog]) -> String {
    let rows: String = logs
        .iter()
        .map(|l| {
            let (left, right) = diff_html(&l.original, &l.corrected);
            format!(
                r#"<tr>
                    <td class="ts">{}</td>
                    <td class="orig">{}</td>
                    <td class="corr">{}</td>
                </tr>"#,
                l.created_at.format("%Y-%m-%d %H:%M:%S"),
                left,
                right,
            )
        })
        .collect();

    format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>neospeller logs</title>
<style>
  :root {{
    color-scheme: dark;
    --bg: #0e1116;
    --panel: #161b22;
    --border: #262d36;
    --fg: #d6dde6;
    --muted: #8a94a3;
    --orig-bg: #2a1518;
    --corr-bg: #11241a;
    --hover: #1c2230;
  }}
  body {{ font: 14px/1.5 -apple-system, system-ui, sans-serif; margin: 2rem; color: var(--fg); background: var(--bg); }}
  h1 {{ font-size: 1.2rem; margin-bottom: 1rem; }}
  .meta {{ color: var(--muted); margin-bottom: 1rem; }}
  table {{ border-collapse: collapse; width: 100%; }}
  th, td {{ text-align: left; padding: 8px 10px; border-bottom: 1px solid var(--border); vertical-align: top; }}
  th {{ background: var(--panel); font-weight: 600; position: sticky; top: 0; }}
  td.ts {{ white-space: nowrap; color: var(--muted); font-variant-numeric: tabular-nums; width: 1%; }}
  td.orig {{ background: var(--orig-bg); }}
  td.corr {{ background: var(--corr-bg); }}
  td.orig, td.corr {{ white-space: pre-wrap; font-family: ui-monospace, SFMono-Regular, Menlo, monospace; font-size: 13px; }}
  .del {{ background: #6e1d24; color: #ffd7d7; border-radius: 2px; padding: 0 2px; }}
  .add {{ background: #1f4a2c; color: #c8f0d4; border-radius: 2px; padding: 0 2px; }}
  tr:hover td {{ background: var(--hover); }}
</style>
</head>
<body>
<h1>neospeller logs</h1>
<div class="meta">{count} entries · newest first</div>
<table>
  <thead>
    <tr><th>when</th><th>original</th><th>corrected</th></tr>
  </thead>
  <tbody>
    {rows}
  </tbody>
</table>
</body>
</html>"#,
        count = logs.len(),
        rows = rows,
    )
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn diff_html(original: &str, corrected: &str) -> (String, String) {
    let diff = TextDiff::from_words(original, corrected);
    let mut left = String::new();
    let mut right = String::new();

    for change in diff.iter_all_changes() {
        let value = escape(change.value());
        match change.tag() {
            ChangeTag::Equal => {
                left.push_str(&value);
                right.push_str(&value);
            }
            ChangeTag::Delete => {
                left.push_str(&format!("<span class=\"del\">{}</span>", value));
            }
            ChangeTag::Insert => {
                right.push_str(&format!("<span class=\"add\">{}</span>", value));
            }
        }
    }

    (left, right)
}
