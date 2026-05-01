use std::time::Duration;

use serde::{Deserialize, Serialize};
use url::Url;

use crate::error::{Error, Result};

/// Browser fingerprint to emulate via [`wreq`]. Mirrors the most useful
/// variants of [`wreq_util::Emulation`] but is serializable so it can live
/// in TOML config files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Emulation {
    Chrome134,
    Chrome135,
    Chrome136,
    Chrome137,
    Firefox136,
    Firefox139,
    Edge134,
    Safari18_3_1,
    SafariIos18_1_1,
    Okhttp5,
}

impl Default for Emulation {
    fn default() -> Self {
        Emulation::Chrome137
    }
}

impl Emulation {
    pub fn to_wreq(self) -> wreq_util::Emulation {
        match self {
            Emulation::Chrome134 => wreq_util::Emulation::Chrome134,
            Emulation::Chrome135 => wreq_util::Emulation::Chrome135,
            Emulation::Chrome136 => wreq_util::Emulation::Chrome136,
            Emulation::Chrome137 => wreq_util::Emulation::Chrome137,
            Emulation::Firefox136 => wreq_util::Emulation::Firefox136,
            Emulation::Firefox139 => wreq_util::Emulation::Firefox139,
            Emulation::Edge134 => wreq_util::Emulation::Edge134,
            Emulation::Safari18_3_1 => wreq_util::Emulation::Safari18_3_1,
            Emulation::SafariIos18_1_1 => wreq_util::Emulation::SafariIos18_1_1,
            Emulation::Okhttp5 => wreq_util::Emulation::OkHttp5,
        }
    }
}

/// How a multi-proxy pool is consumed when scanning many targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProxyStrategy {
    /// One proxy per scan call, advanced atomically.
    #[default]
    RoundRobin,
    /// Random proxy per scan call.
    Random,
    /// Use the same proxy index for the whole `Scanner` lifetime
    /// (selected by URL host hash).
    Sticky,
}

/// Top-level scanner configuration. Construct with [`Config::builder`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub emulation: Emulation,
    pub timeout: Duration,
    pub connect_timeout: Duration,
    pub redirect_limit: usize,
    pub probe_paths: Vec<String>,
    pub max_js_per_probe: usize,
    pub max_js_total: usize,
    pub max_js_bytes: usize,
    pub js_fetch_concurrency: usize,
    pub js_host_blocklist: Vec<String>,
    pub proxies: Vec<String>,
    pub proxy_strategy: ProxyStrategy,
    pub extra_headers: Vec<(String, String)>,
    pub extra_cookies: Vec<(String, String)>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            emulation: Emulation::default(),
            timeout: Duration::from_secs(25),
            connect_timeout: Duration::from_secs(10),
            redirect_limit: 10,
            probe_paths: default_probe_paths(),
            max_js_per_probe: 80,
            max_js_total: 200,
            max_js_bytes: 4 * 1024 * 1024,
            js_fetch_concurrency: 12,
            js_host_blocklist: default_js_host_blocklist(),
            proxies: Vec::new(),
            proxy_strategy: ProxyStrategy::default(),
            extra_headers: Vec::new(),
            extra_cookies: Vec::new(),
        }
    }
}

impl Config {
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }

    pub(crate) fn validate_proxies(&self) -> Result<Vec<Url>> {
        self.proxies
            .iter()
            .map(|p| {
                Url::parse(p).map_err(|e| Error::InvalidProxy {
                    input: p.clone(),
                    source: e,
                })
            })
            .collect()
    }
}

fn default_probe_paths() -> Vec<String> {
    [
        "/",
        "/cart",
        "/basket",
        "/checkout",
        "/shopping-bag",
        "/bag",
        "/payment",
        "/login",
        "/pricing",
        "/subscribe",
        "/billing",
        "/signup",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

fn default_js_host_blocklist() -> Vec<String> {
    [
        "use.typekit.net",
        "p.typekit.net",
        "www.googletagmanager.com",
        "www.google-analytics.com",
        "static.hotjar.com",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

/// Builder for [`Config`]. Every method returns `Self` for chaining.
#[derive(Debug, Default)]
pub struct ConfigBuilder {
    inner: Config,
}

impl ConfigBuilder {
    pub fn emulation(mut self, e: Emulation) -> Self {
        self.inner.emulation = e;
        self
    }

    pub fn timeout(mut self, d: Duration) -> Self {
        self.inner.timeout = d;
        self
    }

    pub fn connect_timeout(mut self, d: Duration) -> Self {
        self.inner.connect_timeout = d;
        self
    }

    pub fn redirect_limit(mut self, n: usize) -> Self {
        self.inner.redirect_limit = n;
        self
    }

    pub fn probe_paths<I, S>(mut self, paths: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.inner.probe_paths = paths.into_iter().map(Into::into).collect();
        self
    }

    pub fn max_js_per_probe(mut self, n: usize) -> Self {
        self.inner.max_js_per_probe = n;
        self
    }

    pub fn max_js_total(mut self, n: usize) -> Self {
        self.inner.max_js_total = n;
        self
    }

    pub fn max_js_bytes(mut self, n: usize) -> Self {
        self.inner.max_js_bytes = n;
        self
    }

    pub fn js_fetch_concurrency(mut self, n: usize) -> Self {
        self.inner.js_fetch_concurrency = n.max(1);
        self
    }

    pub fn js_host_blocklist<I, S>(mut self, hosts: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.inner.js_host_blocklist = hosts.into_iter().map(Into::into).collect();
        self
    }

    pub fn proxy<S: Into<String>>(mut self, proxy: S) -> Self {
        self.inner.proxies = vec![proxy.into()];
        self
    }

    pub fn proxies<I, S>(mut self, proxies: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.inner.proxies = proxies.into_iter().map(Into::into).collect();
        self
    }

    pub fn proxy_strategy(mut self, s: ProxyStrategy) -> Self {
        self.inner.proxy_strategy = s;
        self
    }

    pub fn extra_header<K: Into<String>, V: Into<String>>(mut self, k: K, v: V) -> Self {
        self.inner.extra_headers.push((k.into(), v.into()));
        self
    }

    pub fn extra_cookie<K: Into<String>, V: Into<String>>(mut self, k: K, v: V) -> Self {
        self.inner.extra_cookies.push((k.into(), v.into()));
        self
    }

    pub fn build(self) -> Result<Config> {
        if self.inner.probe_paths.is_empty() {
            return Err(Error::Config(
                "at least one probe path is required".into(),
            ));
        }
        if self.inner.max_js_per_probe == 0 || self.inner.max_js_total == 0 {
            return Err(Error::Config(
                "max_js_per_probe and max_js_total must be > 0".into(),
            ));
        }
        // Validate any configured proxies eagerly so a bad URL surfaces here
        // rather than at first request.
        let _ = self.inner.validate_proxies()?;
        Ok(self.inner)
    }
}
