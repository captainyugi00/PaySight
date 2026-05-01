//! # PaySight Core
//!
//! Detection engine for payment gateways, buy-now-pay-later providers,
//! wallets, bot-mitigation / WAF / CDN systems, captchas, and
//! device-fingerprinting vendors used by arbitrary public websites.
//!
//! PaySight builds an HTTP client with browser-grade TLS / HTTP-2 / JA3
//! fingerprinting (via [`wreq`]), probes a target across a set of common
//! paths, recursively crawls the JavaScript bundles those pages reference,
//! and matches the combined response surface (HTML + headers + cookies + JS)
//! against a curated database of vendor signatures.
//!
//! ## Quick start
//!
//! ```no_run
//! use paysight_core::{Config, Scanner};
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let config = Config::builder().build()?;
//! let scanner = Scanner::new(config)?;
//! let report = scanner.scan("swarovski.com").await?;
//! for hit in &report.payment_hits {
//!     println!("{}: score {}", hit.vendor, hit.score);
//! }
//! # Ok(()) }
//! ```
//!
//! ## Multiple targets
//!
//! ```no_run
//! # use paysight_core::{Config, Scanner};
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let scanner = Scanner::new(Config::builder().build()?)?;
//! let reports = scanner.scan_many(&["stripe.com", "adyen.com"], 4).await;
//! # Ok(()) }
//! ```

pub mod client;
pub mod config;
pub mod detector;
pub mod error;
pub mod report;
pub mod signatures;

pub use config::{Config, ConfigBuilder, Emulation, ProxyStrategy};
pub use detector::{NoopProgress, ProgressSink, Scanner};
pub use error::{Error, Result};
pub use report::{
    AntibotHit, AuthGateStatus, Confidence, CookieFinding, PaymentHit, ProbeOutcome, ScanReport,
};
pub use signatures::{
    AntibotKind, AntibotSignature, PaymentCategory, PaymentSignature,
};
