use std::fs;
use std::process::Command;

use neospeller::firestore_logger::{fetch_all, SpellcheckLog};

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
            format!(
                r#"<tr>
                    <td class="ts">{}</td>
                    <td class="orig">{}</td>
                    <td class="corr">{}</td>
                </tr>"#,
                l.created_at.format("%Y-%m-%d %H:%M:%S"),
                escape(&l.original),
                escape(&l.corrected),
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
  body {{ font: 14px/1.5 -apple-system, system-ui, sans-serif; margin: 2rem; color: #222; }}
  h1 {{ font-size: 1.2rem; margin-bottom: 1rem; }}
  .meta {{ color: #777; margin-bottom: 1rem; }}
  table {{ border-collapse: collapse; width: 100%; }}
  th, td {{ text-align: left; padding: 8px 10px; border-bottom: 1px solid #eee; vertical-align: top; }}
  th {{ background: #fafafa; font-weight: 600; position: sticky; top: 0; }}
  td.ts {{ white-space: nowrap; color: #888; font-variant-numeric: tabular-nums; width: 1%; }}
  td.orig {{ background: #fff5f5; }}
  td.corr {{ background: #f3fbf3; }}
  td.orig, td.corr {{ white-space: pre-wrap; font-family: ui-monospace, SFMono-Regular, Menlo, monospace; font-size: 13px; }}
  tr:hover td {{ background: #f9f9ff; }}
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
