//! Vendor signature database.
//!
//! Each signature contributes a weight per matching pattern. Patterns are
//! matched as case-insensitive literal substrings against the response body,
//! response headers, Set-Cookie names, final URL, and the contents of any
//! crawled JS bundles.
//!
//! Strong, vendor-unique patterns (e.g. CDN-hosted SDK URLs) get higher
//! weights; corroborating identifiers (e.g. compiled JS symbols) get lower
//! weights.

mod payment;
mod protection;

pub use payment::{PaymentCategory, PaymentSignature, PAYMENT_SIGNATURES};
pub use protection::{AntibotKind, AntibotSignature, ANTIBOT_SIGNATURES};

/// A pattern is a literal substring (case-insensitive) plus a weight.
pub type Pattern = (&'static str, u32);
