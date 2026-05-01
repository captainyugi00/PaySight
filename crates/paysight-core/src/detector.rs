use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;

use tokio::sync::Semaphore;
use tracing::{debug, info_span, instrument, trace, Instrument};
use url::Url;
use wreq::Client;

use crate::client::ClientPool;
use crate::config::Config;
use crate::error::{Error, Result};
use crate::report::{
    AntibotHit, AuthGateStatus, Confidence, CookieFinding, PaymentHit, ProbeOutcome, ScanReport,
};
use crate::signatures::{
    AntibotSignature, PaymentCategory, PaymentSignature, ANTIBOT_SIGNATURES, PAYMENT_SIGNATURES,
};

/// Hook called by `Scanner` to report progress while scanning. The CLI
/// uses this to drive `indicatif` progress bars; library users can ignore
/// it. All callbacks must be cheap — they're invoked from the hot path.
pub trait ProgressSink: Send + Sync {
    fn target_started(&self, target: &str);
    fn target_finished(&self, target: &str);
    fn probe_started(&self, target: &str, path: &str);
    fn probe_finished(&self, target: &str, path: &str, status: u16, bytes: usize);
    fn js_bundle_fetched(&self, target: &str, url: &str, bytes: usize);
}

/// No-op sink. Use when you don't care about progress events.
pub struct NoopProgress;
impl ProgressSink for NoopProgress {
    fn target_started(&self, _: &str) {}
    fn target_finished(&self, _: &str) {}
    fn probe_started(&self, _: &str, _: &str) {}
    fn probe_finished(&self, _: &str, _: &str, _: u16, _: usize) {}
    fn js_bundle_fetched(&self, _: &str, _: &str, _: usize) {}
}

/// The high-level scanner. Cheap to clone (everything is `Arc`-backed).
#[derive(Clone)]
pub struct Scanner {
    inner: Arc<ScannerInner>,
}

struct ScannerInner {
    config: Config,
    pool: ClientPool,
}

impl Scanner {
    pub fn new(config: Config) -> Result<Self> {
        let pool = ClientPool::build(&config)?;
        Ok(Self {
            inner: Arc::new(ScannerInner { config, pool }),
        })
    }

    pub fn config(&self) -> &Config {
        &self.inner.config
    }

    pub async fn scan<S: AsRef<str>>(&self, target: S) -> Result<ScanReport> {
        self.scan_with_progress(target, &NoopProgress).await
    }

    #[instrument(skip(self, progress), fields(target = %target.as_ref()))]
    pub async fn scan_with_progress<S: AsRef<str>, P: ProgressSink>(
        &self,
        target: S,
        progress: &P,
    ) -> Result<ScanReport> {
        let target_str = target.as_ref().to_string();
        let base = normalize_target(&target_str)?;
        let host = base.host_str().unwrap_or("").to_string();
        let client = self.inner.pool.pick(&host).clone();
        let started = Instant::now();
        progress.target_started(&target_str);

        let mut probes: Vec<ProbeOutcome> = Vec::new();
        let mut corpus = String::new();
        let mut fetched_js: HashSet<String> = HashSet::new();
        let mut all_set_cookies: Vec<String> = Vec::new();
        let mut final_url = base.to_string();

        for path in &self.inner.config.probe_paths {
            let mut probe_url = base.clone();
            probe_url.set_path(path);
            progress.probe_started(&target_str, path);

            let (mut probe, combined, html_body, set_cookies) =
                fetch_probe(&client, &probe_url).await;
            progress.probe_finished(&target_str, path, probe.status, probe.bytes);

            if !combined.is_empty() {
                corpus.push_str(&combined);
                corpus.push('\n');
            }
            all_set_cookies.extend(set_cookies);

            // Track the deepest final_url from the homepage probe so callers
            // can see where the canonical site landed (after locale redirects
            // etc).
            if path == "/" {
                final_url = probe.final_url.clone();
            }

            // Crawl JS bundles from this probe (parallel, bounded).
            if probe.error.is_none()
                && !html_body.is_empty()
                && fetched_js.len() < self.inner.config.max_js_total
            {
                let final_base =
                    Url::parse(&probe.final_url).unwrap_or_else(|_| probe_url.clone());
                let resources = extract_resource_urls(&html_body, &final_base);
                let mut to_fetch: Vec<Url> = Vec::new();
                for resource in resources {
                    if to_fetch.len() >= self.inner.config.max_js_per_probe
                        || fetched_js.len() + to_fetch.len() >= self.inner.config.max_js_total
                    {
                        break;
                    }
                    if !worth_fetching(&resource, &self.inner.config.js_host_blocklist) {
                        continue;
                    }
                    let key = resource.to_string();
                    if !fetched_js.insert(key) {
                        continue;
                    }
                    to_fetch.push(resource);
                }
                let scanned = fetch_js_parallel(
                    &client,
                    to_fetch,
                    &mut corpus,
                    self.inner.config.js_fetch_concurrency,
                    self.inner.config.max_js_bytes,
                    &target_str,
                    progress,
                )
                .await;
                probe.js_bundles_scanned = scanned;
            }

            probes.push(probe);
        }

        let corpus_lower = corpus.to_lowercase();
        let payment_hits = scan_payment(&corpus_lower, PAYMENT_SIGNATURES);
        let antibot_hits = scan_antibots(&corpus_lower, ANTIBOT_SIGNATURES);
        let cookies = extract_cookie_names(&all_set_cookies);
        let auth_gate = classify_auth_gate(&probes, &payment_hits);

        progress.target_finished(&target_str);

        Ok(ScanReport {
            target: target_str,
            final_url,
            started_at: now_iso8601(),
            elapsed_ms: started.elapsed().as_millis(),
            probes,
            payment_hits,
            antibot_hits,
            cookies,
            auth_gate,
        })
    }

    /// Scan multiple targets concurrently. `parallelism` is the maximum
    /// number of in-flight scans. Failures are reported per-target.
    pub async fn scan_many<S: AsRef<str>>(
        &self,
        targets: &[S],
        parallelism: usize,
    ) -> Vec<(String, Result<ScanReport>)> {
        self.scan_many_with_progress(targets, parallelism, Arc::new(NoopProgress))
            .await
    }

    pub async fn scan_many_with_progress<S, P>(
        &self,
        targets: &[S],
        parallelism: usize,
        progress: Arc<P>,
    ) -> Vec<(String, Result<ScanReport>)>
    where
        S: AsRef<str>,
        P: ProgressSink + 'static,
    {
        let sem = Arc::new(Semaphore::new(parallelism.max(1)));
        let mut handles = Vec::with_capacity(targets.len());
        for t in targets {
            let target = t.as_ref().to_string();
            let scanner = self.clone();
            let sem = sem.clone();
            let progress = progress.clone();
            handles.push(tokio::spawn(
                async move {
                    let _permit = sem.acquire_owned().await.ok();
                    let res = scanner.scan_with_progress(&target, progress.as_ref()).await;
                    (target, res)
                }
                .instrument(info_span!("scan_target")),
            ));
        }
        let mut out = Vec::with_capacity(handles.len());
        for h in handles {
            match h.await {
                Ok(pair) => out.push(pair),
                Err(e) => out.push((
                    String::from("<panic>"),
                    Err(Error::ClientBuild(format!("task panicked: {e}"))),
                )),
            }
        }
        out
    }
}

fn normalize_target(input: &str) -> Result<Url> {
    let trimmed = input.trim();
    let with_scheme = if trimmed.contains("://") {
        trimmed.to_string()
    } else {
        format!("https://{}", trimmed)
    };
    Url::parse(&with_scheme).map_err(|e| Error::InvalidTarget {
        input: input.to_string(),
        source: e,
    })
}

async fn fetch_probe(
    client: &Client,
    url: &Url,
) -> (ProbeOutcome, String, String, Vec<String>) {
    let url_str = url.to_string();
    match client.get(url.clone()).send().await {
        Ok(resp) => {
            let final_url = resp.url().to_string();
            let status = resp.status().as_u16();
            let mut header_blob = String::new();
            let mut server_header = None;
            let mut set_cookies: Vec<String> = Vec::new();
            for (k, v) in resp.headers().iter() {
                let key = k.as_str();
                if let Ok(val) = v.to_str() {
                    header_blob.push_str(key);
                    header_blob.push_str(": ");
                    header_blob.push_str(val);
                    header_blob.push('\n');
                    if key.eq_ignore_ascii_case("server") && server_header.is_none() {
                        server_header = Some(val.to_string());
                    }
                    if key.eq_ignore_ascii_case("set-cookie") {
                        set_cookies.push(val.to_string());
                    }
                }
            }
            let redirected_to_auth = looks_like_auth_redirect(&final_url);
            match resp.text().await {
                Ok(body) => {
                    let combined = format!(
                        "{}\n---HEADERS---\n{}\n---URL---\n{}",
                        body, header_blob, final_url
                    );
                    let probe = ProbeOutcome {
                        url: url_str,
                        final_url,
                        status,
                        bytes: combined.len(),
                        js_bundles_scanned: 0,
                        server_header,
                        error: None,
                        redirected_to_auth,
                    };
                    (probe, combined, body, set_cookies)
                }
                Err(e) => {
                    let probe = ProbeOutcome {
                        url: url_str,
                        final_url,
                        status,
                        bytes: 0,
                        js_bundles_scanned: 0,
                        server_header,
                        error: Some(format!("body read failed: {e}")),
                        redirected_to_auth,
                    };
                    (probe, String::new(), String::new(), set_cookies)
                }
            }
        }
        Err(e) => {
            let probe = ProbeOutcome {
                url: url_str.clone(),
                final_url: url_str,
                status: 0,
                bytes: 0,
                js_bundles_scanned: 0,
                server_header: None,
                error: Some(format!("request failed: {e}")),
                redirected_to_auth: false,
            };
            (probe, String::new(), String::new(), Vec::new())
        }
    }
}

fn looks_like_auth_redirect(url: &str) -> bool {
    let lower = url.to_lowercase();
    [
        "/login",
        "/signin",
        "/sign-in",
        "/signup",
        "/sign-up",
        "/auth",
        "/account",
        "/member/signup",
        "/oauth",
        "/sso",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

async fn fetch_js_one(client: &Client, url: &Url, max_bytes: usize) -> Option<String> {
    let resp = client.get(url.clone()).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let text = resp.text().await.ok()?;
    if text.len() > max_bytes {
        Some(text[..max_bytes].to_string())
    } else {
        Some(text)
    }
}

async fn fetch_js_parallel<P: ProgressSink>(
    client: &Client,
    urls: Vec<Url>,
    corpus: &mut String,
    concurrency: usize,
    max_bytes: usize,
    target: &str,
    progress: &P,
) -> usize {
    use tokio::task::JoinSet;
    let mut set: JoinSet<Option<(String, String)>> = JoinSet::new();
    let mut iter = urls.into_iter();
    for _ in 0..concurrency.max(1) {
        if let Some(url) = iter.next() {
            let c = client.clone();
            set.spawn(async move {
                let key = url.to_string();
                fetch_js_one(&c, &url, max_bytes).await.map(|s| (key, s))
            });
        }
    }
    let mut scanned = 0usize;
    while let Some(res) = set.join_next().await {
        if let Some(url) = iter.next() {
            let c = client.clone();
            set.spawn(async move {
                let key = url.to_string();
                fetch_js_one(&c, &url, max_bytes).await.map(|s| (key, s))
            });
        }
        if let Ok(Some((key, js))) = res {
            progress.js_bundle_fetched(target, &key, js.len());
            corpus.push_str("\n---JS:");
            corpus.push_str(&key);
            corpus.push_str("---\n");
            corpus.push_str(&js);
            corpus.push('\n');
            scanned += 1;
        }
    }
    scanned
}

fn extract_resource_urls(html: &str, base: &Url) -> Vec<Url> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    for (key, attr) in &[("<script", "src"), ("<link", "href")] {
        let bytes = html.as_bytes();
        let key_bytes = key.as_bytes();
        let mut idx = 0;
        while idx + key_bytes.len() < bytes.len() {
            if let Some(pos) = find_subslice(&bytes[idx..], key_bytes) {
                let tag_start = idx + pos;
                if let Some(close) = find_subslice(&bytes[tag_start..], b">") {
                    let tag = &html[tag_start..tag_start + close];
                    if let Some(src) = extract_attr(tag, attr) {
                        if !src.is_empty() {
                            if let Ok(resolved) = base.join(&src) {
                                if seen.insert(resolved.to_string()) {
                                    out.push(resolved);
                                }
                            }
                        }
                    }
                    idx = tag_start + close + 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }
    out
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    haystack
        .windows(needle.len())
        .position(|window| window.eq_ignore_ascii_case(needle))
}

fn extract_attr(tag: &str, attr: &str) -> Option<String> {
    let lower = tag.to_lowercase();
    let needle = format!("{attr}=");
    let pos = lower.find(&needle)?;
    let rest = &tag[pos + needle.len()..];
    let bytes = rest.as_bytes();
    let (quote, start) = match bytes.first()? {
        b'"' => (b'"', 1),
        b'\'' => (b'\'', 1),
        _ => return None,
    };
    let after = &rest[start..];
    let end = after.bytes().position(|b| b == quote)?;
    Some(after[..end].to_string())
}

fn worth_fetching(candidate: &Url, blocklist: &[String]) -> bool {
    if candidate.scheme() != "https" && candidate.scheme() != "http" {
        return false;
    }
    let path = candidate.path().to_lowercase();
    if !path.ends_with(".js") {
        return false;
    }
    let host = candidate.host_str().unwrap_or("").to_lowercase();
    if blocklist.iter().any(|h| host == h.as_str()) {
        return false;
    }
    true
}

fn extract_cookie_names(set_cookie_lines: &[String]) -> Vec<CookieFinding> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for line in set_cookie_lines {
        if let Some(eq) = line.find('=') {
            let name = line[..eq].trim().to_string();
            if !name.is_empty() && seen.insert(name.clone()) {
                out.push(CookieFinding { name });
            }
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

fn scan_payment(corpus_lower: &str, signatures: &[PaymentSignature]) -> Vec<PaymentHit> {
    let mut hits: HashMap<&'static str, PaymentHit> = HashMap::new();
    for sig in signatures {
        for (pat, weight) in sig.patterns {
            let needle = pat.to_lowercase();
            if corpus_lower.contains(&needle) {
                let entry = hits.entry(sig.vendor).or_insert_with(|| PaymentHit {
                    vendor: sig.vendor.to_string(),
                    slug: sig.slug.to_string(),
                    category: sig.category,
                    score: 0,
                    confidence: Confidence::None,
                    matched_patterns: Vec::new(),
                });
                let pat_owned = (*pat).to_string();
                if !entry.matched_patterns.contains(&pat_owned) {
                    entry.score += weight;
                    entry.matched_patterns.push(pat_owned);
                }
            }
        }
    }
    let mut out: Vec<PaymentHit> = hits.into_values().collect();
    for h in &mut out {
        h.confidence = Confidence::from_score(h.score);
    }
    out.sort_by(|a, b| b.score.cmp(&a.score));
    trace!(hit_count = out.len(), "payment scan complete");
    out
}

fn scan_antibots(corpus_lower: &str, signatures: &[AntibotSignature]) -> Vec<AntibotHit> {
    let mut hits: HashMap<&'static str, AntibotHit> = HashMap::new();
    for sig in signatures {
        for (pat, weight) in sig.patterns {
            let needle = pat.to_lowercase();
            if corpus_lower.contains(&needle) {
                let entry = hits.entry(sig.vendor).or_insert_with(|| AntibotHit {
                    vendor: sig.vendor.to_string(),
                    slug: sig.slug.to_string(),
                    kind: sig.kind,
                    score: 0,
                    confidence: Confidence::None,
                    matched_patterns: Vec::new(),
                });
                let pat_owned = (*pat).to_string();
                if !entry.matched_patterns.contains(&pat_owned) {
                    entry.score += weight;
                    entry.matched_patterns.push(pat_owned);
                }
            }
        }
    }
    let mut out: Vec<AntibotHit> = hits.into_values().collect();
    for h in &mut out {
        h.confidence = Confidence::from_score(h.score);
    }
    out.sort_by(|a, b| b.score.cmp(&a.score));
    trace!(hit_count = out.len(), "antibot scan complete");
    out
}

fn classify_auth_gate(probes: &[ProbeOutcome], payment_hits: &[PaymentHit]) -> AuthGateStatus {
    let checkout_paths = ["/cart", "/basket", "/checkout", "/shopping-bag", "/bag"];
    let checkout_probes: Vec<&ProbeOutcome> = probes
        .iter()
        .filter(|p| {
            let path_url = Url::parse(&p.url).ok();
            let path = path_url.as_ref().map(|u| u.path().to_string()).unwrap_or_default();
            checkout_paths.iter().any(|cp| path == *cp)
        })
        .collect();

    if checkout_probes.is_empty() {
        return AuthGateStatus::Unknown;
    }

    let any_open_checkout_loaded = checkout_probes
        .iter()
        .any(|p| p.status == 200 && !p.redirected_to_auth);
    let has_primary_gateway = payment_hits
        .iter()
        .any(|h| h.category == PaymentCategory::PrimaryGateway && h.score >= 5);

    let all_blocked = checkout_probes.iter().all(|p| {
        p.redirected_to_auth || p.status == 401 || p.status == 403 || p.status == 404
    });

    match (any_open_checkout_loaded, has_primary_gateway, all_blocked) {
        (_, true, _) => AuthGateStatus::Open,
        (false, false, true) => AuthGateStatus::Gated,
        _ => {
            debug!("ambiguous auth-gate signals — marking unknown");
            AuthGateStatus::Unknown
        }
    }
}

fn now_iso8601() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = now.as_secs();
    // Format as RFC 3339 manually to avoid pulling in chrono just for this.
    let (y, mo, d, h, mi, se) = epoch_to_ymdhms(secs);
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{mi:02}:{se:02}Z")
}

fn epoch_to_ymdhms(secs: u64) -> (i32, u32, u32, u32, u32, u32) {
    let days = secs / 86_400;
    let rem = secs % 86_400;
    let h = (rem / 3600) as u32;
    let mi = ((rem % 3600) / 60) as u32;
    let se = (rem % 60) as u32;

    // Howard Hinnant's civil-from-days algorithm.
    let z = days as i64 + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let mo = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = if mo <= 2 { y + 1 } else { y };
    (y as i32, mo, d, h, mi, se)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confidence_buckets() {
        assert_eq!(Confidence::from_score(0), Confidence::None);
        assert_eq!(Confidence::from_score(3), Confidence::Weak);
        assert_eq!(Confidence::from_score(7), Confidence::Moderate);
        assert_eq!(Confidence::from_score(15), Confidence::Strong);
        assert_eq!(Confidence::from_score(25), Confidence::VeryStrong);
    }

    #[test]
    fn auth_redirect_detection() {
        assert!(looks_like_auth_redirect("https://x.com/login"));
        assert!(looks_like_auth_redirect("https://x.com/member/signup/select_type"));
        assert!(!looks_like_auth_redirect("https://x.com/checkout"));
    }

    #[test]
    fn extract_attr_basic() {
        assert_eq!(extract_attr("<script src=\"a.js\">", "src"), Some("a.js".into()));
        assert_eq!(extract_attr("<script src='a.js'>", "src"), Some("a.js".into()));
        assert_eq!(extract_attr("<script defer>", "src"), None);
    }

    #[test]
    fn pattern_scan_picks_up_signal() {
        let corpus = "stuff before js.stripe.com/v3 after".to_lowercase();
        let hits = scan_payment(&corpus, PAYMENT_SIGNATURES);
        let stripe = hits.iter().find(|h| h.vendor == "Stripe").unwrap();
        assert!(stripe.score >= 6);
    }
}
