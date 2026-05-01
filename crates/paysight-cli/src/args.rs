use std::path::PathBuf;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use clap::{Parser, ValueEnum};
use paysight_core::{Config, Emulation, ProxyStrategy};
use serde::Deserialize;

/// PaySight: scan websites for payment, BNPL, wallet, captcha, fingerprint
/// and bot-mitigation vendors. Renders text, JSON, or PDF reports.
#[derive(Debug, Parser)]
#[command(
    name = "paysight",
    version,
    author,
    about = "Payment + protection stack reconnaissance",
    long_about = None,
    propagate_version = true,
)]
pub struct Cli {
    /// One or more target hostnames or URLs (e.g. `swarovski.com`,
    /// `https://stripe.com`). Combine with `--targets-file` to add more.
    #[arg(value_name = "TARGET")]
    pub targets: Vec<String>,

    /// Read additional targets from a file (one per line, `#` comments allowed).
    #[arg(long, value_name = "FILE")]
    pub targets_file: Option<PathBuf>,

    /// Path to TOML configuration file. CLI flags override values from the file.
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Output format.
    #[arg(long, default_value = "text", value_enum)]
    pub format: OutputFormat,

    /// Write output to this file (required for `--format pdf`).
    #[arg(long, short = 'o', value_name = "FILE")]
    pub out: Option<PathBuf>,

    // --- Scanning behaviour ---

    /// Browser emulation profile.
    #[arg(long, value_enum)]
    pub emulation: Option<EmulationArg>,

    /// Comma-separated list of probe paths (overrides defaults).
    #[arg(long, value_delimiter = ',', value_name = "PATH")]
    pub probe_paths: Option<Vec<String>>,

    /// Maximum JS bundles to fetch per probe page.
    #[arg(long, value_name = "N")]
    pub max_js_per_probe: Option<usize>,

    /// Maximum JS bundles to fetch overall per target.
    #[arg(long, value_name = "N")]
    pub max_js_total: Option<usize>,

    /// Cap each JS bundle at this many bytes.
    #[arg(long, value_name = "BYTES")]
    pub max_js_bytes: Option<usize>,

    /// Concurrent JS bundle fetches per probe.
    #[arg(long, value_name = "N")]
    pub concurrency: Option<usize>,

    /// Number of targets to scan in parallel.
    #[arg(long, value_name = "N", default_value_t = 4)]
    pub parallel: usize,

    /// Per-request timeout (e.g. `25s`, `1m`).
    #[arg(long, value_name = "DURATION")]
    pub timeout: Option<humantime::Duration>,

    /// TCP connect timeout.
    #[arg(long, value_name = "DURATION")]
    pub connect_timeout: Option<humantime::Duration>,

    /// Max redirects to follow.
    #[arg(long, value_name = "N")]
    pub redirect_limit: Option<usize>,

    // --- Proxy ---

    /// Single proxy URL (HTTP / HTTPS / SOCKS5). Use `--proxies-file` for a pool.
    #[arg(long, value_name = "URL", env = "PAYSIGHT_PROXY")]
    pub proxy: Option<String>,

    /// Read a proxy pool from a file (one URL per line).
    #[arg(long, value_name = "FILE")]
    pub proxies_file: Option<PathBuf>,

    /// How to consume the proxy pool when present.
    #[arg(long, value_enum, default_value = "round-robin")]
    pub proxy_strategy: ProxyStrategyArg,

    // --- Headers / cookies ---

    /// Extra request header in `Name: Value` form (repeatable).
    #[arg(long = "header", short = 'H', value_name = "HEADER", action = clap::ArgAction::Append)]
    pub headers: Vec<String>,

    /// Extra cookie in `name=value` form (repeatable).
    #[arg(long = "cookie", short = 'C', value_name = "COOKIE", action = clap::ArgAction::Append)]
    pub cookies: Vec<String>,

    // --- UI ---

    /// Suppress the ASCII banner.
    #[arg(long)]
    pub no_banner: bool,

    /// Disable color and animation.
    #[arg(long)]
    pub no_color: bool,

    /// Suppress all progress UI.
    #[arg(long, short = 'q')]
    pub quiet: bool,

    /// Increase log verbosity (-v info, -vv debug, -vvv trace).
    #[arg(long, short = 'v', action = clap::ArgAction::Count)]
    pub verbose: u8,
}

#[derive(Debug, Copy, Clone, ValueEnum)]
#[value(rename_all = "snake_case")]
pub enum OutputFormat {
    Text,
    Json,
    Html,
}

#[derive(Debug, Copy, Clone, ValueEnum)]
#[value(rename_all = "snake_case")]
pub enum EmulationArg {
    Chrome134,
    Chrome135,
    Chrome136,
    Chrome137,
    Firefox136,
    Firefox139,
    Edge134,
    Safari1831,
    SafariIos1811,
    Okhttp5,
}

impl From<EmulationArg> for Emulation {
    fn from(e: EmulationArg) -> Self {
        match e {
            EmulationArg::Chrome134 => Emulation::Chrome134,
            EmulationArg::Chrome135 => Emulation::Chrome135,
            EmulationArg::Chrome136 => Emulation::Chrome136,
            EmulationArg::Chrome137 => Emulation::Chrome137,
            EmulationArg::Firefox136 => Emulation::Firefox136,
            EmulationArg::Firefox139 => Emulation::Firefox139,
            EmulationArg::Edge134 => Emulation::Edge134,
            EmulationArg::Safari1831 => Emulation::Safari18_3_1,
            EmulationArg::SafariIos1811 => Emulation::SafariIos18_1_1,
            EmulationArg::Okhttp5 => Emulation::Okhttp5,
        }
    }
}

#[derive(Debug, Copy, Clone, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub enum ProxyStrategyArg {
    RoundRobin,
    Random,
    Sticky,
}

impl From<ProxyStrategyArg> for ProxyStrategy {
    fn from(p: ProxyStrategyArg) -> Self {
        match p {
            ProxyStrategyArg::RoundRobin => ProxyStrategy::RoundRobin,
            ProxyStrategyArg::Random => ProxyStrategy::Random,
            ProxyStrategyArg::Sticky => ProxyStrategy::Sticky,
        }
    }
}

/// Subset of [`Config`] expressible in TOML. Used to merge file values with
/// CLI overrides.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileConfig {
    pub emulation: Option<Emulation>,
    pub timeout_secs: Option<u64>,
    pub connect_timeout_secs: Option<u64>,
    pub redirect_limit: Option<usize>,
    pub probe_paths: Option<Vec<String>>,
    pub max_js_per_probe: Option<usize>,
    pub max_js_total: Option<usize>,
    pub max_js_bytes: Option<usize>,
    pub js_fetch_concurrency: Option<usize>,
    pub js_host_blocklist: Option<Vec<String>>,
    pub proxies: Option<Vec<String>>,
    pub proxy_strategy: Option<ProxyStrategy>,
    pub extra_headers: Option<Vec<(String, String)>>,
    pub extra_cookies: Option<Vec<(String, String)>>,
}

/// Merge a TOML file (if any) with the CLI args into a [`Config`].
pub fn build_config(cli: &Cli) -> Result<Config> {
    let file_config: FileConfig = match &cli.config {
        Some(path) => {
            let text = std::fs::read_to_string(path)
                .with_context(|| format!("reading config file {}", path.display()))?;
            toml::from_str(&text)
                .with_context(|| format!("parsing config file {}", path.display()))?
        }
        None => FileConfig::default(),
    };

    let mut builder = Config::builder();

    if let Some(e) = cli.emulation {
        builder = builder.emulation(e.into());
    } else if let Some(e) = file_config.emulation {
        builder = builder.emulation(e);
    }

    if let Some(d) = cli.timeout {
        builder = builder.timeout((*d).into());
    } else if let Some(secs) = file_config.timeout_secs {
        builder = builder.timeout(Duration::from_secs(secs));
    }

    if let Some(d) = cli.connect_timeout {
        builder = builder.connect_timeout((*d).into());
    } else if let Some(secs) = file_config.connect_timeout_secs {
        builder = builder.connect_timeout(Duration::from_secs(secs));
    }

    if let Some(n) = cli.redirect_limit.or(file_config.redirect_limit) {
        builder = builder.redirect_limit(n);
    }

    if let Some(paths) = cli
        .probe_paths
        .clone()
        .or(file_config.probe_paths.clone())
    {
        builder = builder.probe_paths(paths);
    }

    if let Some(n) = cli.max_js_per_probe.or(file_config.max_js_per_probe) {
        builder = builder.max_js_per_probe(n);
    }
    if let Some(n) = cli.max_js_total.or(file_config.max_js_total) {
        builder = builder.max_js_total(n);
    }
    if let Some(n) = cli.max_js_bytes.or(file_config.max_js_bytes) {
        builder = builder.max_js_bytes(n);
    }
    if let Some(n) = cli.concurrency.or(file_config.js_fetch_concurrency) {
        builder = builder.js_fetch_concurrency(n);
    }
    if let Some(blocklist) = file_config.js_host_blocklist {
        builder = builder.js_host_blocklist(blocklist);
    }

    // Proxy resolution: CLI single proxy > CLI file > config file.
    let mut proxies: Vec<String> = Vec::new();
    if let Some(p) = &cli.proxy {
        proxies.push(p.clone());
    }
    if let Some(path) = &cli.proxies_file {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("reading proxies file {}", path.display()))?;
        for line in text.lines() {
            let l = line.trim();
            if !l.is_empty() && !l.starts_with('#') {
                proxies.push(l.to_string());
            }
        }
    }
    if proxies.is_empty() {
        if let Some(list) = file_config.proxies {
            proxies = list;
        }
    }
    if !proxies.is_empty() {
        builder = builder.proxies(proxies);
    }

    let strategy = file_config
        .proxy_strategy
        .unwrap_or_else(|| cli.proxy_strategy.into());
    builder = builder.proxy_strategy(strategy);

    // Extra headers
    for raw in &cli.headers {
        let (k, v) = raw
            .split_once(':')
            .ok_or_else(|| anyhow!("invalid --header `{raw}` (expected `Name: Value`)"))?;
        builder = builder.extra_header(k.trim(), v.trim());
    }
    if let Some(list) = file_config.extra_headers {
        for (k, v) in list {
            builder = builder.extra_header(k, v);
        }
    }

    // Extra cookies
    for raw in &cli.cookies {
        let (k, v) = raw
            .split_once('=')
            .ok_or_else(|| anyhow!("invalid --cookie `{raw}` (expected `name=value`)"))?;
        builder = builder.extra_cookie(k.trim(), v.trim());
    }
    if let Some(list) = file_config.extra_cookies {
        for (k, v) in list {
            builder = builder.extra_cookie(k, v);
        }
    }

    Ok(builder.build()?)
}

/// Resolve all targets from positional args and `--targets-file`.
pub fn collect_targets(cli: &Cli) -> Result<Vec<String>> {
    let mut out: Vec<String> = cli.targets.clone();
    if let Some(path) = &cli.targets_file {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("reading targets file {}", path.display()))?;
        for line in text.lines() {
            let l = line.trim();
            if !l.is_empty() && !l.starts_with('#') {
                out.push(l.to_string());
            }
        }
    }
    Ok(out)
}
