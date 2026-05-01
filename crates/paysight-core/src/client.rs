use std::sync::atomic::{AtomicUsize, Ordering};

use tracing::debug;
use url::Url;
use wreq::redirect::Policy;
use wreq::{Client, Proxy};

use crate::config::{Config, ProxyStrategy};
use crate::error::{Error, Result};

/// A pool of `wreq::Client`s — one per configured proxy, or a single client
/// with no proxy. Used to round-robin / shard requests across proxies.
pub(crate) struct ClientPool {
    clients: Vec<Client>,
    cursor: AtomicUsize,
    strategy: ProxyStrategy,
}

impl ClientPool {
    pub fn build(config: &Config) -> Result<Self> {
        let proxies = config.validate_proxies()?;
        let clients = if proxies.is_empty() {
            vec![build_client(config, None)?]
        } else {
            proxies
                .iter()
                .map(|p| build_client(config, Some(p)))
                .collect::<Result<Vec<_>>>()?
        };
        debug!(client_count = clients.len(), strategy = ?config.proxy_strategy, "client pool ready");
        Ok(Self {
            clients,
            cursor: AtomicUsize::new(0),
            strategy: config.proxy_strategy,
        })
    }

    /// Pick a client for a given target host. Implementation depends on
    /// the configured strategy.
    pub fn pick(&self, target_host: &str) -> &Client {
        if self.clients.len() == 1 {
            return &self.clients[0];
        }
        let n = self.clients.len();
        let idx = match self.strategy {
            ProxyStrategy::RoundRobin => self.cursor.fetch_add(1, Ordering::Relaxed) % n,
            ProxyStrategy::Random => fastrand_index(n),
            ProxyStrategy::Sticky => host_hash(target_host) % n,
        };
        &self.clients[idx]
    }

    pub fn size(&self) -> usize {
        self.clients.len()
    }
}

impl std::fmt::Debug for ClientPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClientPool")
            .field("size", &self.size())
            .field("strategy", &self.strategy)
            .finish()
    }
}

fn build_client(config: &Config, proxy_url: Option<&Url>) -> Result<Client> {
    let mut builder = Client::builder()
        .emulation(config.emulation.to_wreq())
        .redirect(Policy::limited(config.redirect_limit))
        .timeout(config.timeout)
        .connect_timeout(config.connect_timeout);

    if let Some(p) = proxy_url {
        let proxy = Proxy::all(p.as_str())
            .map_err(|e| Error::ClientBuild(format!("invalid proxy `{p}`: {e}")))?;
        builder = builder.proxy(proxy);
    }

    builder
        .build()
        .map_err(|e| Error::ClientBuild(e.to_string()))
}

/// Tiny non-secret hash for sticky proxy assignment. We don't need
/// cryptographic strength here — just stable bucketing per host.
fn host_hash(host: &str) -> usize {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in host.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h as usize
}

/// Tiny LCG-based RNG to avoid pulling in a `rand` dependency just for
/// proxy selection. Seeded from `Instant::now()` once per process via the
/// thread-local cell.
fn fastrand_index(n: usize) -> usize {
    use std::cell::Cell;
    use std::time::{SystemTime, UNIX_EPOCH};
    thread_local! {
        static STATE: Cell<u64> = const { Cell::new(0) };
    }
    STATE.with(|s| {
        let mut v = s.get();
        if v == 0 {
            v = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(0x9E3779B97F4A7C15)
                | 1;
        }
        // xorshift64
        v ^= v << 13;
        v ^= v >> 7;
        v ^= v << 17;
        s.set(v);
        (v as usize) % n
    })
}
