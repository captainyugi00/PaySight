//! # PaySight Report
//!
//! Renders [`paysight_core::ScanReport`] values as JSON or HTML documents.
//!
//! - JSON: pretty-printed via `serde_json`.
//! - HTML: a single self-contained page with embedded vendor logos, the
//!   PaySight gradient brand, animated reveals, and a print-friendly
//!   stylesheet. No assets are loaded at view time except for the Inter
//!   and JetBrains Mono fonts (Google Fonts CDN).
//!
//! ## Example
//!
//! ```no_run
//! use paysight_core::ScanReport;
//!
//! # fn make() -> Vec<ScanReport> { vec![] }
//! let reports = make();
//! paysight_report::render_html(&reports, "report.html").unwrap();
//! ```

mod assets;
mod html;

use paysight_core::ScanReport;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReportError {
    #[error("serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("HTML render error: {0}")]
    Html(String),
}

/// Serialize one or more reports as pretty-printed JSON.
pub fn render_json(reports: &[ScanReport]) -> Result<String, ReportError> {
    Ok(serde_json::to_string_pretty(reports)?)
}

/// Render reports to a single self-contained HTML file at `out`.
pub fn render_html<P: AsRef<Path>>(reports: &[ScanReport], out: P) -> Result<(), ReportError> {
    html::render_html(reports, out)
}

/// Same as [`render_html`] but returns the HTML as a `String`.
pub fn render_html_string(reports: &[ScanReport]) -> String {
    html::render_html_string(reports)
}
