# Architecture

PaySight is a Cargo workspace with three crates and an asset tree.

```
PaySight/
├── Cargo.toml                    # workspace root
├── README.md
├── LICENSE                       # Apache-2.0
├── rust-toolchain.toml           # pin: stable
├── crates/
│   ├── paysight-core/            # async SDK: detection engine
│   │   └── src/
│   │       ├── lib.rs            # public re-exports
│   │       ├── error.rs          # thiserror::Error
│   │       ├── config.rs         # Config + ConfigBuilder + Emulation + ProxyStrategy
│   │       ├── client.rs         # ClientPool — wreq clients per proxy
│   │       ├── detector.rs       # Scanner + scan/scan_many + ProgressSink
│   │       ├── report.rs         # ScanReport + PaymentHit + AntibotHit + ProbeOutcome
│   │       └── signatures/
│   │           ├── mod.rs
│   │           ├── payment.rs    # PAYMENT_SIGNATURES
│   │           └── protection.rs # ANTIBOT_SIGNATURES
│   ├── paysight-report/          # JSON + HTML renderers
│   │   └── src/
│   │       ├── lib.rs            # render_json + render_html
│   │       ├── html.rs           # HTML template assembly
│   │       ├── assets.rs         # include_bytes! for logos + CSS + JS
│   │       ├── report.css        # one self-contained stylesheet
│   │       └── report.js         # IntersectionObserver reveals
│   └── paysight-cli/             # `paysight` binary
│       └── src/
│           ├── main.rs           # entry, format dispatch
│           ├── args.rs           # clap derive + TOML config merge
│           ├── banner.rs         # gradient ASCII banner
│           ├── progress.rs       # indicatif MultiProgressSink
│           └── text_output.rs    # comfy-table colored tables
├── assets/
│   └── logos/{payment,protection}/*.svg     # vendor logos (Simple Icons CC0 + placeholder)
├── tools/
│   └── fetch_logos.sh            # idempotent simple-icons fetcher
├── docs/
│   ├── ARCHITECTURE.md           # this file
│   └── previews/                 # screenshots used by the README
└── examples/
    └── basic.rs                  # SDK walk-through
```

## Detection pipeline

```
target string ─► normalize ─► ClientPool::pick(host) ─► wreq::Client
                                                               │
                                                               ▼
                                          for path in PROBE_PATHS:
                                              fetch HTML + headers + cookies
                                              extract <script src> / <link href>
                                              fetch every JS bundle (parallel, bounded)
                                          accumulate corpus
                                                               │
                                                               ▼
                                  scan corpus.to_lowercase()  against:
                                      PAYMENT_SIGNATURES   (35 vendors)
                                      ANTIBOT_SIGNATURES   (22 vendors)
                                                               │
                                                               ▼
                                       classify auth-gate from probe redirects
                                                               │
                                                               ▼
                                                          ScanReport
```

### Probe paths

Default: `/`, `/cart`, `/basket`, `/checkout`, `/shopping-bag`, `/bag`, `/payment`, `/login`, `/pricing`, `/subscribe`, `/billing`, `/signup`. Override with `Config::probe_paths` or `--probe-paths`.

### JS bundle crawl

Each probe page yields a list of `<script src>` and `<link href>` resolved URLs. Filtering:

- Only `.js` files
- Excluded hosts: typekit.net, googletagmanager.com, google-analytics.com, hotjar.com (configurable).

Up to `max_js_per_probe` (default 80) are fetched per probe page, capped at `max_js_total` overall (default 200). `js_fetch_concurrency` (default 12) sets the in-flight limit per probe via `tokio::task::JoinSet`. Each bundle is read up to `max_js_bytes` (4 MiB) and appended to the scan corpus.

### Signature schema

```rust
pub struct PaymentSignature {
    pub vendor: &'static str,
    pub slug: &'static str,                   // logo lookup key
    pub category: PaymentCategory,
    pub patterns: &'static [(&'static str, u32)],
}
```

Each `(pattern, weight)` is matched as a case-insensitive literal substring against the corpus. A vendor's score is the sum of distinct matched-pattern weights. Confidence buckets:

| Score | Bucket |
|---|---|
| 0 | none |
| 1–4 | weak |
| 5–9 | moderate |
| 10–19 | strong |
| 20+ | very strong |

Patterns should be **stable, unique, and observable on the public surface** (or in compiled JS bundles). Examples that work well:

- Vendor-hosted CDN URLs: `js.stripe.com`, `checkoutshopper-live.adyen.com`
- Globally unique cookie names: `__cf_bm`, `datadome=`, `_abck=`
- Vendor-specific JS API tokens: `AdyenCheckout(`, `Razorpay(`, `Frames.init`
- Compiled JS symbols: `hasAdyenPaymentsBlock`, `fingerprintjs2`

Avoid patterns that match generic substrings ("api", "cdn", "checkout") — false positives across the corpus.

### Auth-gate classification

A target is `gated` when every checkout-class probe (`/cart`, `/basket`, `/checkout`, `/shopping-bag`, `/bag`) either redirects to a login/signup path or returns 401/403/404 **and** no primary-gateway signature was matched. It is `open` when at least one of those probes loaded successfully *or* a primary-gateway hit was scored. Otherwise `unknown`.

## Adding a new vendor

1. **Pick a slug.** Lowercase, no spaces, matches the simple-icons name when possible (e.g. `stripe`, `cloudflare`, `adyen`).
2. **Add a signature** to `crates/paysight-core/src/signatures/payment.rs` or `protection.rs`. Pick weights conservatively — a single pattern of weight 6 implies "this vendor is *here*"; smaller weights are corroborating.
3. **Add the logo.** Run `tools/fetch_logos.sh`. If the slug isn't in simple-icons it will fall back to the placeholder shield. You can also drop a hand-crafted SVG into `assets/logos/{payment,protection}/<slug>.svg`.
4. **Wire the slug into `paysight-report/src/assets.rs`** — add a match arm in `payment_logo` or `protection_logo`. (`include_bytes!` is per-vendor for compile-time string-literal requirements.)
5. **Test.** `cargo test --release` runs unit tests on the signature scan; the `pattern_scan_picks_up_signal` test in `detector.rs` shows the shape.
6. **Run a real scan** through a known target and confirm the vendor is identified at the expected confidence band.

## Proxy support

`paysight-core::Config::proxies` is a `Vec<String>`. URLs accept `http://`, `https://`, and `socks5://` schemes (the `socks` feature of `wreq` is enabled by default). Each proxy URL builds its own `wreq::Client` (`Arc`-shared internally), and `ClientPool` selects one per scan call:

- **`round-robin`** (default) — `AtomicUsize` cursor, advanced once per `scan` call.
- **`random`** — xorshift-seeded thread-local RNG.
- **`sticky`** — FNV-1a hash of the target host modulo pool size; same target always uses the same proxy.

A scan call uses a single proxy for *all* its probes (so cookies / TLS fingerprints stay coherent), then the next scan picks the next.

## Output renderers

`paysight-report` is decoupled from the engine. It only consumes `Vec<ScanReport>`:

```rust
let reports: Vec<ScanReport> = ...;
paysight_report::render_json(&reports)?;          // String
paysight_report::render_html(&reports, "out.html")?;  // file
```

Both formats are stable and serializable; downstream tooling can parse the JSON directly.
