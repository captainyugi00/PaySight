//! Basic SDK walk-through.
//!
//! Builds a [`Scanner`] with a tuned [`Config`], scans three targets in
//! parallel, prints a one-liner per target, then renders a polished HTML
//! report to `target/example-report.html`.
//!
//! Run with:
//!
//! ```sh
//! cargo run --release --example basic
//! ```

use std::time::Duration;

use paysight_core::{Config, Emulation, Scanner};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::builder()
        .emulation(Emulation::Chrome137)
        .timeout(Duration::from_secs(20))
        .max_js_per_probe(60)
        .js_fetch_concurrency(8)
        .build()?;

    let scanner = Scanner::new(config)?;
    let targets = ["swarovski.com", "cursor.com", "sephora.com"];

    let pairs = scanner.scan_many(&targets, 3).await;
    let mut reports = Vec::new();
    for (target, res) in pairs {
        match res {
            Ok(report) => {
                let primary = report
                    .primary_gateway()
                    .map(|h| format!("{} ({})", h.vendor, h.confidence.label()))
                    .unwrap_or_else(|| "no primary gateway".to_string());
                println!(
                    "{} → {} · {}",
                    target,
                    primary,
                    report.auth_gate.label()
                );
                reports.push(report);
            }
            Err(e) => eprintln!("{target} → error: {e}"),
        }
    }

    paysight_report::render_html(&reports, "target/example-report.html")?;
    println!("\nWrote target/example-report.html");
    Ok(())
}
