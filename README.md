# PaySight

> Payment + protection stack reconnaissance.
> Scan any website and get back the payment processor, BNPL provider, wallet, captcha, WAF/CDN, bot manager, and device-fingerprinting vendors it uses — with confidence-scored evidence and a polished HTML / JSON / terminal report.

PaySight is a Rust workspace that ships:

| Crate | Purpose |
|---|---|
| **`paysight-core`** | Async detection engine. Browser-grade TLS / HTTP-2 / JA3 fingerprinting via [`wreq`], curated signature DB, parallel target scanning, proxy pool support. |
| **`paysight-report`** | Self-contained HTML + JSON renderers. Embedded vendor logos, animated reveals, score bars, dark mode. |
| **`paysight-cli`** | The `paysight` command-line tool. Multi-target scans, animated multi-progress, color-coded tables, gradient banner. |

---

## Preview

<p align="center">
  <img src="docs/previews/report-dark.png" alt="PaySight HTML report — dark mode" width="640">
</p>

A single self-contained HTML report. Gradient brand, glassy cards, vendor logos inlined as SVG, score bars that animate on scroll, and an auth-gated badge that pulses when a target's checkout is login-walled.

### CLI

```text
▎ superdry.com → https://www.superdry.com/
  4698 ms  12 probes  [checkout open]
  PRIMARY Adyen ([moderate] · score 9)

  [Payment stack]
╭───────────────────┬───────────────────────────┬───────┬────────────┬────────────────────────────────────────╮
│ Category          ┆ Vendor                    ┆ Score ┆ Confidence ┆ Patterns                               │
╞═══════════════════╪═══════════════════════════╪═══════╪════════════╪════════════════════════════════════════╡
│ Commerce platform ┆ Salesforce Commerce Cloud ┆    10 ┆ strong     ┆ demandware.static, on/demandware.store │
│ Primary gateway   ┆ Adyen                     ┆     9 ┆ moderate   ┆ AdyenPayments, hasAdyenPaymentsBlock   │
╰───────────────────┴───────────────────────────┴───────┴────────────┴────────────────────────────────────────╯

  [Bot mitigation / WAF / CDN]
╭─────────┬────────────┬───────┬────────────┬───────────────────────────────────────────────╮
│ Kind    ┆ Vendor     ┆ Score ┆ Confidence ┆ Patterns                                      │
╞═════════╪════════════╪═══════╪════════════╪═══════════════════════════════════════════════╡
│ CDN     ┆ Cloudflare ┆    16 ┆ strong     ┆ cf-ray:, server: cloudflare, cf-cache-status: │
│ Captcha ┆ reCAPTCHA  ┆     5 ┆ moderate   ┆ grecaptcha.execute                            │
╰─────────┴────────────┴───────┴────────────┴───────────────────────────────────────────────╯

  [Cookies set during probes]
     __cq_dnt   dw_dnt   dwanonymous_b2999f30149a47649334f32c757a86ce   dwsid   sid
```

In a real terminal: gradient ASCII banner, animated multi-target spinners, color-coded confidence pills, and a pulsing badge for auth-gated checkouts.

---

## Highlights

- **Browser-grade requests.** Each scan goes out with a real Chrome 137 / Safari 18 / Firefox 139 TLS, HTTP-2, and JA3 fingerprint via `wreq` + BoringSSL. Sites that 403 generic curl / reqwest clients return real content.
- **Layered detection.** ~35 payment-vendor signatures (Stripe, Adyen, Braintree, Worldpay, Checkout.com, Klarna, Afterpay, PayPal, Apple/Google/Amazon Pay, Cardinal Commerce 3DS …) and ~22 protection-vendor signatures (Cloudflare, Akamai Bot Manager, DataDome, Imperva, PerimeterX, Kasada, Shape, Arkose FunCaptcha, FingerprintJS, Forter, Riskified, Signifyd …).
- **Deep crawl.** Probes 12 URL paths per target (cart / checkout / login / pricing / etc), pulls every JS bundle they reference (up to 200, 12 concurrent), and matches signatures against the combined HTML + headers + cookies + JS corpus.
- **Auth-gate detection.** When a target's checkout SDK lives behind login, the report explicitly flags the site as auth-gated and explains why.
- **Proxy pool.** HTTP / HTTPS / SOCKS5 with auth, single proxy or pool. Pool consumed via round-robin, random, or sticky-by-host.
- **Three output modes.** Animated terminal output (`--format text`), pretty JSON (`--format json`), or self-contained HTML report with a gradient brand, embedded SVG logos, score-bar reveals, and a print-friendly fallback (`--format html`).

---

## Install

```sh
git clone https://github.com/captainyugi00/PaySight
cd PaySight
cargo build --release
./target/release/paysight --help
```

Requires Rust 1.85+ and `cmake` (for BoringSSL) at build time. On macOS:

```sh
brew install rust cmake
```

---

## CLI usage

```sh
# Single target, animated terminal output
paysight swarovski.com

# Multiple targets in parallel
paysight superdry.com swarovski.com sephora.com cursor.com --parallel 4

# Read targets from a file (one per line, # comments allowed)
paysight --targets-file targets.txt --parallel 8

# Polished HTML report
paysight swarovski.com cursor.com --format html --out report.html

# JSON for downstream tooling
paysight stripe.com --format json --out stripe.json

# Run through a proxy pool
paysight swarovski.com \
  --proxies-file proxies.txt \
  --proxy-strategy round-robin

# A single SOCKS5 proxy with auth
paysight swarovski.com --proxy "socks5://user:pass@proxy.example:1080"

# Custom probe paths and JS crawl budget
paysight myshop.com \
  --probe-paths "/,/shop/cart,/checkout,/account/login" \
  --max-js-per-probe 120 \
  --max-js-total 400 \
  --concurrency 16 \
  --timeout 30s
```

Full flag list: `paysight --help`.

### Configuration file

Every CLI flag has a TOML equivalent. CLI overrides file values.

```toml
# paysight.toml
emulation = "chrome137"
timeout_secs = 25
connect_timeout_secs = 10
redirect_limit = 10
probe_paths = ["/", "/cart", "/checkout", "/login"]
max_js_per_probe = 80
max_js_total = 200
js_fetch_concurrency = 12

proxies = [
    "http://user:pass@proxy1.example:8080",
    "socks5://proxy2.example:1080",
]
proxy_strategy = "sticky"

extra_headers = [["X-Test", "yes"]]
extra_cookies = [["preview", "abc123"]]
```

```sh
paysight --config paysight.toml swarovski.com
```

---

## SDK usage

```toml
[dependencies]
paysight-core   = { git = "https://github.com/captainyugi00/PaySight" }
paysight-report = { git = "https://github.com/captainyugi00/PaySight" }
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

```rust
use paysight_core::{Config, Scanner};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::builder()
        .timeout(std::time::Duration::from_secs(20))
        .max_js_per_probe(60)
        .proxies(["socks5://localhost:1080"])
        .build()?;

    let scanner = Scanner::new(config)?;
    let report = scanner.scan("swarovski.com").await?;

    if let Some(top) = report.primary_gateway() {
        println!("primary gateway: {} ({})", top.vendor, top.confidence.label());
    }

    paysight_report::render_html(&[report], "swarovski.html")?;
    Ok(())
}
```

See [`crates/paysight-cli/examples/basic.rs`](crates/paysight-cli/examples/basic.rs) for a more thorough walk-through.

---

## Output: HTML report

The HTML output is a single self-contained file (~60 KB plus inlined SVGs):

- Gradient PaySight wordmark with a slow shimmer animation
- One card per target with a backdrop-filter glass effect that lifts on hover
- Animated score bars that fill on scroll (Intersection Observer)
- Auth-gated badges pulse to call attention to login-walled flows
- Inline monochrome vendor logos (`currentColor` — pick up the theme)
- Dark mode by default with `prefers-color-scheme: light` fallback
- Print-friendly stylesheet for `Cmd-P` → PDF
- Inter + JetBrains Mono via Google Fonts

---

## Architecture

See [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) for crate boundaries, signature schema, and how to add a new vendor.

---

## Acknowledgements

- TLS fingerprinting: [wreq](https://github.com/0x676e67/wreq) — modular Rust HTTP client with browser-grade emulation.
- Vendor logos: [Simple Icons](https://github.com/simple-icons/simple-icons) (CC0). Vendors not in the catalog use a neutral monochrome shield.
- Typefaces: [Inter](https://rsms.me/inter/) and [JetBrains Mono](https://www.jetbrains.com/lp/mono/) via Google Fonts CDN.

---

## Disclaimers & responsible use

> **Read this before running PaySight against any system you don't own.**

### Authorized use only

PaySight is a reconnaissance tool. It issues ordinary HTTP(S) requests (one homepage probe + a handful of well-known cart / checkout / login / pricing paths, plus the JavaScript bundles those pages reference) and matches the response surface against a static signature database. It does **not** authenticate, submit forms, follow purchase flows, exploit vulnerabilities, or attempt to evade WAFs, bot managers, or captchas.

That said, automated requests against a service you don't own may still be governed by:

- the target's **Terms of Service** and `robots.txt` policy,
- the **Computer Fraud and Abuse Act** (US) and equivalent statutes in your jurisdiction (e.g. UK Computer Misuse Act, EU NIS2, DE §202c StGB, FR Loi Godfrain, etc.),
- **GDPR / CCPA** and other data-protection regimes if the responses contain personal data,
- contractual restrictions imposed by your **proxy provider**,
- export-control rules if you operate from / route through embargoed jurisdictions.

You are solely responsible for ensuring you have the necessary authorization before scanning. Examples of legitimate use:

- Scanning sites **you own or operate**.
- Bug-bounty programs whose **scope explicitly permits** this kind of recon.
- CTF and educational labs where the target is a deliberately published challenge.
- Internal vendor-stack inventory of your own properties.
- Academic / journalistic research conducted in line with applicable research-ethics frameworks.

If you're not sure whether your use case is permitted, **don't run PaySight against that target**.

### Detection accuracy

The signature database is a curated set of heuristics — substring matches on URLs, JS symbols, header names, and Set-Cookie names that, in our experience, are correlated with each vendor's presence. It is:

- **Not authoritative.** Vendors rename SDKs, ship new CDN domains, and obfuscate bundles. False positives and false negatives are inevitable.
- **Public-surface only.** Many checkout / payment / 3-D Secure SDKs only load *after* authentication or with cart contents. PaySight will mark such sites `auth-gated` rather than guess.
- **A starting point, not a conclusion.** Use the matched-pattern list to verify findings yourself before publishing or making business decisions.

### No bypass, no exploitation

- PaySight uses browser-grade TLS / HTTP-2 / JA3 fingerprinting via [wreq] **purely so the target sees a request consistent with a real browser visit** — i.e. so it doesn't 403 a generic SDK client. It does not solve captchas, replay challenges, lift `__cf_bm` / `_abck` / `datadome` cookies, or interact with any anti-bot challenge surface.
- It does **not** probe authenticated routes, exploit vulnerabilities, fuzz inputs, or perform any form of credential stuffing or account takeover.
- The proxy-pool feature is provided for legitimate geo-distribution (e.g. confirming a vendor only ships in a specific market) and for working around rate-limits on **your own** infrastructure.

### Rate-limit & network etiquette

A single scan issues roughly 12 page fetches and up to ~200 JS-bundle fetches against one origin (configurable). Repeated full-concurrency runs against the same site without consent may:

- Trip the target's bot-defense thresholds and pollute their telemetry.
- Be billable to the target as bandwidth.
- In extreme cases be classified as a denial-of-service attempt.

Default to one scan per target per session. If you need to run sweeps, add `--timeout`, throttle `--parallel`, and stagger runs.

### Privacy

JSON / HTML reports may contain:

- Response headers (`Server`, `CF-Ray`, `Set-Cookie` lines)
- Cookie names set by the target (token names, not token values)
- The final URL after redirects (which can leak the target's locale / A/B bucket / session identifier)

Don't share reports of someone else's site publicly without a quick review.

### Trademarks & logos

All vendor names, product names, and logos referenced in this repository — including but not limited to Stripe, Adyen, Braintree, Worldpay, Checkout.com, PayPal, Apple Pay, Google Pay, Amazon Pay, Klarna, Afterpay, Affirm, Cardinal Commerce, Cloudflare, Akamai, AWS, DataDome, PerimeterX, Imperva, Kasada, Arkose Labs, FingerprintJS, Forter, Riskified, Signifyd — are **trademarks of their respective owners**. Their use here is **nominative fair use** for the purpose of identifying the vendor a target site appears to be using. PaySight is **not affiliated with, endorsed by, or sponsored by any of these vendors**.

Logos shipped under `assets/logos/` come from the [Simple Icons](https://github.com/simple-icons/simple-icons) project (CC0 1.0 Universal Public Domain Dedication) where coverage exists; vendors not in the Simple Icons catalog use a neutral monochrome shield. If a vendor's brand owner objects to inclusion, open an issue and we'll remove or replace the asset.

### Card data

PaySight detects payment-authentication providers (e.g. Cardinal Commerce / Songbird, EMV 3-D Secure) by their **public JS markers only**. It never sees, transmits, or stores cardholder data, and is not in scope for PCI-DSS.

### No warranty

This software is provided **"AS IS", without warranty of any kind**, express or implied, including but not limited to the warranties of merchantability, fitness for a particular purpose, non-infringement, and accuracy of detection. The authors and contributors are not liable for any claim, damages, or other liability arising from the use of, or inability to use, this software. See the [Apache-2.0 License](LICENSE) for the full terms.

By using PaySight you agree to bear all risk of running it against any target.

---

## License

[Apache-2.0](LICENSE).

[`wreq`]: https://github.com/0x676e67/wreq
