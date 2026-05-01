use std::fmt;

use serde::{Deserialize, Serialize};

use crate::signatures::{AntibotKind, PaymentCategory};

/// Bucketed confidence band for a hit, derived from its weighted score.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    None,
    Weak,
    Moderate,
    Strong,
    VeryStrong,
}

impl Confidence {
    pub fn from_score(score: u32) -> Self {
        match score {
            0 => Confidence::None,
            1..=4 => Confidence::Weak,
            5..=9 => Confidence::Moderate,
            10..=19 => Confidence::Strong,
            _ => Confidence::VeryStrong,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Confidence::None => "no match",
            Confidence::Weak => "weak",
            Confidence::Moderate => "moderate",
            Confidence::Strong => "strong",
            Confidence::VeryStrong => "very strong",
        }
    }
}

impl fmt::Display for Confidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentHit {
    pub vendor: String,
    pub slug: String,
    pub category: PaymentCategory,
    pub score: u32,
    pub confidence: Confidence,
    pub matched_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntibotHit {
    pub vendor: String,
    pub slug: String,
    pub kind: AntibotKind,
    pub score: u32,
    pub confidence: Confidence,
    pub matched_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeOutcome {
    pub url: String,
    pub final_url: String,
    pub status: u16,
    pub bytes: usize,
    pub js_bundles_scanned: usize,
    pub server_header: Option<String>,
    pub error: Option<String>,
    /// True if this probe redirected to a login / signup / signin page —
    /// strong indicator that the underlying flow is auth-gated.
    pub redirected_to_auth: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookieFinding {
    pub name: String,
}

/// Indicates how confident we are that a target's checkout is auth-gated
/// (i.e. the actual payment SDK does not load on public surfaces).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthGateStatus {
    /// At least one cart/checkout-class probe loaded successfully and we
    /// observed payment-vendor signatures.
    Open,
    /// All cart/checkout probes redirected to login/signup or 4xx'd —
    /// the payment SDK is almost certainly behind auth.
    Gated,
    /// Mixed signals or no checkout-class probes loaded.
    Unknown,
}

impl AuthGateStatus {
    pub fn label(self) -> &'static str {
        match self {
            AuthGateStatus::Open => "open",
            AuthGateStatus::Gated => "auth-gated",
            AuthGateStatus::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanReport {
    pub target: String,
    pub final_url: String,
    pub started_at: String,
    pub elapsed_ms: u128,
    pub probes: Vec<ProbeOutcome>,
    pub payment_hits: Vec<PaymentHit>,
    pub antibot_hits: Vec<AntibotHit>,
    pub cookies: Vec<CookieFinding>,
    pub auth_gate: AuthGateStatus,
}

impl ScanReport {
    /// Top primary-gateway hit (sorted by score desc), if any.
    pub fn primary_gateway(&self) -> Option<&PaymentHit> {
        self.payment_hits
            .iter()
            .filter(|h| h.category == PaymentCategory::PrimaryGateway)
            .max_by_key(|h| h.score)
    }
}
