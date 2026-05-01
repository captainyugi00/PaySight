//! Animated multi-progress UI driven by [`indicatif`].
//!
//! The `MultiProgressSink` implements `paysight_core::ProgressSink`. Each
//! target gets its own spinner that updates in place as probes and JS
//! bundle fetches stream in. When the target finishes, the spinner is
//! finalized to a static line summarizing the result.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;

use console::Style;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use paysight_core::ProgressSink;

pub struct MultiProgressSink {
    multi: MultiProgress,
    bars: Mutex<HashMap<String, ProgressBar>>,
    counts: Mutex<HashMap<String, BarCounts>>,
}

#[derive(Default)]
struct BarCounts {
    probes_done: usize,
    js_done: usize,
    bytes: u64,
}

impl MultiProgressSink {
    pub fn new() -> Self {
        Self {
            multi: MultiProgress::new(),
            bars: Mutex::new(HashMap::new()),
            counts: Mutex::new(HashMap::new()),
        }
    }

    pub fn multi(&self) -> &MultiProgress {
        &self.multi
    }

    fn ensure_bar(&self, target: &str) -> ProgressBar {
        let mut bars = self.bars.lock().unwrap();
        if let Some(b) = bars.get(target) {
            return b.clone();
        }
        let bar = self.multi.add(ProgressBar::new_spinner());
        bar.set_style(
            ProgressStyle::with_template(
                "  {spinner:.cyan.bold} {prefix:<28.bold} {wide_msg:.dim}",
            )
            .unwrap()
            .tick_strings(&[
                "⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏",
            ]),
        );
        bar.set_prefix(target.to_string());
        bar.enable_steady_tick(Duration::from_millis(80));
        bar.set_message("queued");
        bars.insert(target.to_string(), bar.clone());
        self.counts
            .lock()
            .unwrap()
            .insert(target.to_string(), BarCounts::default());
        bar
    }
}

impl Default for MultiProgressSink {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressSink for MultiProgressSink {
    fn target_started(&self, target: &str) {
        let bar = self.ensure_bar(target);
        bar.set_message("scanning…");
    }

    fn target_finished(&self, target: &str) {
        if let Some(bar) = self.bars.lock().unwrap().get(target) {
            let counts = self.counts.lock().unwrap();
            let c = counts.get(target).cloned_default();
            let check = Style::new().green().bold().apply_to("✓");
            let summary = format!(
                "{} probes · {} js · {} fetched",
                c.probes_done,
                c.js_done,
                fmt_bytes(c.bytes)
            );
            bar.set_style(
                ProgressStyle::with_template("  {prefix:<28.bold} {msg}")
                    .unwrap(),
            );
            bar.set_message(format!("{check} {}", Style::new().dim().apply_to(summary)));
            bar.finish();
        }
    }

    fn probe_started(&self, target: &str, path: &str) {
        if let Some(bar) = self.bars.lock().unwrap().get(target) {
            bar.set_message(format!("→ {path}"));
        }
    }

    fn probe_finished(&self, target: &str, _path: &str, _status: u16, _bytes: usize) {
        if let Some(c) = self.counts.lock().unwrap().get_mut(target) {
            c.probes_done += 1;
        }
    }

    fn js_bundle_fetched(&self, target: &str, url: &str, bytes: usize) {
        let mut counts = self.counts.lock().unwrap();
        if let Some(c) = counts.get_mut(target) {
            c.js_done += 1;
            c.bytes += bytes as u64;
        }
        let counts_snapshot = counts.get(target).cloned_default();
        drop(counts);
        if let Some(bar) = self.bars.lock().unwrap().get(target) {
            // Show the host of the bundle being scanned so users see the JS
            // crawl moving across CDNs.
            let short = short_origin(url);
            bar.set_message(format!(
                "{} js · {} · {}",
                counts_snapshot.js_done,
                fmt_bytes(counts_snapshot.bytes),
                short
            ));
        }
    }
}

trait ClonedDefault {
    fn cloned_default(self) -> BarCounts;
}

impl ClonedDefault for Option<&BarCounts> {
    fn cloned_default(self) -> BarCounts {
        match self {
            Some(c) => BarCounts {
                probes_done: c.probes_done,
                js_done: c.js_done,
                bytes: c.bytes,
            },
            None => BarCounts::default(),
        }
    }
}

fn fmt_bytes(b: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    if b >= MB {
        format!("{:.1} MB", b as f64 / MB as f64)
    } else if b >= KB {
        format!("{:.0} KB", b as f64 / KB as f64)
    } else {
        format!("{b} B")
    }
}

fn short_origin(url: &str) -> String {
    if let Ok(parsed) = url::Url::parse(url) {
        parsed.host_str().unwrap_or("").to_string()
    } else {
        url.split('/').nth(2).unwrap_or(url).to_string()
    }
}
