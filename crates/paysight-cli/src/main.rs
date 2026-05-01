mod args;
mod banner;
mod progress;
mod text_output;

use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use console::Term;
use paysight_core::{ProgressSink, Scanner};
use tracing_subscriber::EnvFilter;

use crate::args::{Cli, OutputFormat};
use crate::progress::MultiProgressSink;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    init_tracing(cli.verbose);

    let colorful = !cli.no_color && Term::stdout().is_term();
    if !cli.no_banner && matches!(cli.format, OutputFormat::Text) {
        banner::print(colorful);
    }

    let targets = args::collect_targets(&cli)?;
    if targets.is_empty() {
        return Err(anyhow!(
            "no targets specified. Pass at least one hostname, or use --targets-file."
        ));
    }

    let config = args::build_config(&cli)?;
    let scanner = Scanner::new(config)?;

    // Decide whether to show animated progress.
    let show_progress =
        !cli.quiet && colorful && matches!(cli.format, OutputFormat::Text);

    let reports = if show_progress {
        let sink = Arc::new(MultiProgressSink::new());
        let progress = sink.clone() as Arc<dyn ProgressSinkDyn>;
        // Build a small adapter so the trait object satisfies the
        // generic bound on `scan_many_with_progress`.
        let adapter = Arc::new(SinkAdapter(sink.clone()));
        let pairs = scanner
            .scan_many_with_progress(&targets, cli.parallel, adapter)
            .await;
        let _ = progress; // kept alive until here
        sink.multi().clear().ok();
        collect_reports(pairs)?
    } else {
        let pairs = scanner.scan_many(&targets, cli.parallel).await;
        collect_reports(pairs)?
    };

    match cli.format {
        OutputFormat::Text => {
            for report in &reports {
                text_output::render(report, colorful);
            }
        }
        OutputFormat::Json => {
            let json = paysight_report::render_json(&reports)?;
            if let Some(path) = &cli.out {
                std::fs::write(path, json.as_bytes())
                    .with_context(|| format!("writing JSON to {}", path.display()))?;
                eprintln!("wrote JSON: {}", path.display());
            } else {
                println!("{json}");
            }
        }
        OutputFormat::Html => {
            let path = cli
                .out
                .as_ref()
                .ok_or_else(|| anyhow!("--out is required for --format html"))?;
            paysight_report::render_html(&reports, path)
                .with_context(|| format!("rendering HTML to {}", path.display()))?;
            eprintln!("wrote HTML: {}", path.display());
        }
    }

    Ok(())
}

fn init_tracing(verbosity: u8) {
    let default = match verbosity {
        0 => "warn",
        1 => "paysight_core=info,paysight_cli=info",
        2 => "paysight_core=debug,paysight_cli=debug",
        _ => "paysight_core=trace,paysight_cli=trace",
    };
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| default.into());
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_writer(std::io::stderr)
        .try_init();
}

fn collect_reports(
    pairs: Vec<(String, paysight_core::Result<paysight_core::ScanReport>)>,
) -> Result<Vec<paysight_core::ScanReport>> {
    let mut out = Vec::with_capacity(pairs.len());
    for (target, res) in pairs {
        match res {
            Ok(r) => out.push(r),
            Err(e) => eprintln!("⚠ scan failed for {target}: {e}"),
        }
    }
    Ok(out)
}

// --- Trait-object plumbing for the progress sink ------------------------
//
// `Scanner::scan_many_with_progress` is generic over `P: ProgressSink + 'static`.
// We want to be able to drop in different concrete sinks, so we wrap the
// `Arc<MultiProgressSink>` in an adapter type and hand that in.

trait ProgressSinkDyn: Send + Sync {}
impl ProgressSinkDyn for MultiProgressSink {}

struct SinkAdapter(Arc<MultiProgressSink>);

impl ProgressSink for SinkAdapter {
    fn target_started(&self, t: &str) {
        self.0.target_started(t)
    }
    fn target_finished(&self, t: &str) {
        self.0.target_finished(t)
    }
    fn probe_started(&self, t: &str, p: &str) {
        self.0.probe_started(t, p)
    }
    fn probe_finished(&self, t: &str, p: &str, s: u16, b: usize) {
        self.0.probe_finished(t, p, s, b)
    }
    fn js_bundle_fetched(&self, t: &str, u: &str, b: usize) {
        self.0.js_bundle_fetched(t, u, b)
    }
}
