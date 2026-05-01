use serde::{Deserialize, Serialize};

use super::Pattern;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AntibotKind {
    Cdn,
    Waf,
    BotManager,
    Captcha,
    Fingerprint,
}

impl AntibotKind {
    pub fn label(self) -> &'static str {
        match self {
            AntibotKind::Cdn => "CDN",
            AntibotKind::Waf => "WAF",
            AntibotKind::BotManager => "Bot manager",
            AntibotKind::Captcha => "Captcha",
            AntibotKind::Fingerprint => "Fingerprint / device intel",
        }
    }

    pub fn slug(self) -> &'static str {
        match self {
            AntibotKind::Cdn => "cdn",
            AntibotKind::Waf => "waf",
            AntibotKind::BotManager => "bot_manager",
            AntibotKind::Captcha => "captcha",
            AntibotKind::Fingerprint => "fingerprint",
        }
    }
}

pub struct AntibotSignature {
    pub vendor: &'static str,
    pub slug: &'static str,
    pub kind: AntibotKind,
    pub patterns: &'static [Pattern],
}

pub static ANTIBOT_SIGNATURES: &[AntibotSignature] = &[
    AntibotSignature {
        vendor: "Cloudflare",
        slug: "cloudflare",
        kind: AntibotKind::Cdn,
        patterns: &[
            ("cf-ray:", 6),
            ("server: cloudflare", 6),
            ("__cf_bm", 6),
            ("cf_clearance", 6),
            ("__cflb", 5),
            ("cf-cache-status:", 4),
            ("cdn-cgi/challenge-platform", 6),
            ("cdn-cgi/bm/cv", 5),
            ("cf-mitigated:", 6),
        ],
    },
    AntibotSignature {
        vendor: "Cloudflare Turnstile",
        slug: "cloudflareturnstile",
        kind: AntibotKind::Captcha,
        patterns: &[
            ("challenges.cloudflare.com/turnstile", 6),
            ("turnstile.callback", 5),
        ],
    },
    AntibotSignature {
        vendor: "Akamai",
        slug: "akamai",
        kind: AntibotKind::Cdn,
        patterns: &[
            ("server: akamaighost", 6),
            ("akamaihd.net", 4),
            ("x-akamai-", 5),
            ("akamai-edge", 4),
        ],
    },
    AntibotSignature {
        vendor: "Akamai Bot Manager",
        slug: "akamaibotmanager",
        kind: AntibotKind::BotManager,
        patterns: &[
            ("_abck=", 6),
            ("ak_bmsc=", 6),
            ("bm_sz=", 6),
            ("bm_sv=", 5),
            ("bm_so=", 5),
            ("bm_mi=", 5),
            ("akam/", 4),
            ("/_bm/", 4),
        ],
    },
    AntibotSignature {
        vendor: "AWS CloudFront",
        slug: "awscloudfront",
        kind: AntibotKind::Cdn,
        patterns: &[
            ("cloudfront.net", 4),
            ("x-amz-cf-id:", 5),
            ("x-amz-cf-pop:", 5),
            ("x-cache: hit from cloudfront", 4),
            ("x-cache: miss from cloudfront", 4),
        ],
    },
    AntibotSignature {
        vendor: "AWS WAF",
        slug: "awswaf",
        kind: AntibotKind::Waf,
        patterns: &[
            ("aws-waf-token=", 6),
            ("x-amzn-waf-", 6),
            ("/aws-waf/captcha", 5),
        ],
    },
    AntibotSignature {
        vendor: "Fastly",
        slug: "fastly",
        kind: AntibotKind::Cdn,
        patterns: &[
            ("x-served-by: cache-", 5),
            ("x-fastly-", 5),
            ("via: 1.1 varnish", 4),
            ("fastly.net", 3),
        ],
    },
    AntibotSignature {
        vendor: "Imperva (Incapsula)",
        slug: "imperva",
        kind: AntibotKind::BotManager,
        patterns: &[
            ("incap_ses_", 6),
            ("visid_incap_", 6),
            ("nlbi_", 5),
            ("x-iinfo:", 6),
            ("incapsula", 5),
            ("_incapsula_resource", 6),
        ],
    },
    AntibotSignature {
        vendor: "DataDome",
        slug: "datadome",
        kind: AntibotKind::BotManager,
        patterns: &[
            ("datadome=", 6),
            ("x-datadome:", 6),
            ("x-datadome-cid:", 6),
            ("js.datadome.co", 6),
            ("captcha-delivery.com", 5),
            ("dd_cookie_test", 4),
            ("datadome.co/captcha/", 6),
        ],
    },
    AntibotSignature {
        vendor: "PerimeterX / HUMAN",
        slug: "perimeterx",
        kind: AntibotKind::BotManager,
        patterns: &[
            ("_px2=", 6),
            ("_px3=", 6),
            ("_pxhd=", 6),
            ("_pxvid=", 6),
            ("pxcts=", 5),
            ("client.perimeterx.net", 6),
            ("px-cdn.net", 6),
            ("px-cloud.net", 5),
            ("/_pxhc/", 4),
        ],
    },
    AntibotSignature {
        vendor: "Kasada",
        slug: "kasada",
        kind: AntibotKind::BotManager,
        patterns: &[("kp_uidz", 6), ("x-kpsdk-", 6), ("kasada", 5)],
    },
    AntibotSignature {
        vendor: "Shape Security (F5)",
        slug: "shape",
        kind: AntibotKind::BotManager,
        patterns: &[
            ("x-acf-sensor-data", 6),
            ("/_/sensor", 5),
            ("shapesecurity", 5),
            ("f5-shape", 5),
        ],
    },
    AntibotSignature {
        vendor: "reCAPTCHA",
        slug: "recaptcha",
        kind: AntibotKind::Captcha,
        patterns: &[
            ("google.com/recaptcha/api.js", 6),
            ("gstatic.com/recaptcha", 5),
            ("grecaptcha.execute", 5),
            ("data-recaptcha", 3),
        ],
    },
    AntibotSignature {
        vendor: "hCaptcha",
        slug: "hcaptcha",
        kind: AntibotKind::Captcha,
        patterns: &[
            ("hcaptcha.com/1/api.js", 6),
            ("js.hcaptcha.com", 6),
            ("hcaptcha.execute", 5),
        ],
    },
    AntibotSignature {
        vendor: "GeeTest",
        slug: "geetest",
        kind: AntibotKind::Captcha,
        patterns: &[
            ("static.geetest.com", 6),
            ("geevisit.com", 5),
            ("geetest.init", 5),
        ],
    },
    AntibotSignature {
        vendor: "Arkose Labs FunCaptcha",
        slug: "arkose",
        kind: AntibotKind::Captcha,
        patterns: &[
            ("funcaptcha", 5),
            ("arkoselabs.com", 6),
            ("client-api.arkoselabs.com", 6),
        ],
    },
    AntibotSignature {
        vendor: "FingerprintJS",
        slug: "fingerprintjs",
        kind: AntibotKind::Fingerprint,
        patterns: &[
            ("fpjscdn.net", 6),
            ("fingerprint.com/api", 5),
            ("fpjs-cdn", 5),
            ("FingerprintJS", 5),
            ("fingerprintjs2", 5),
            ("Fingerprint2", 4),
        ],
    },
    AntibotSignature {
        vendor: "Sift",
        slug: "sift",
        kind: AntibotKind::Fingerprint,
        patterns: &[
            ("cdn.siftscience.com", 6),
            ("sift_session_id", 4),
            ("_sift", 3),
        ],
    },
    AntibotSignature {
        vendor: "ThreatMetrix",
        slug: "threatmetrix",
        kind: AntibotKind::Fingerprint,
        patterns: &[
            ("h.online-metrix.net", 6),
            ("threatmetrix", 5),
            ("tmx_profiling", 5),
        ],
    },
    AntibotSignature {
        vendor: "Forter",
        slug: "forter",
        kind: AntibotKind::Fingerprint,
        patterns: &[
            ("cdn4.forter.com", 6),
            ("forter.com/script", 5),
            ("forterToken", 5),
        ],
    },
    AntibotSignature {
        vendor: "Riskified",
        slug: "riskified",
        kind: AntibotKind::Fingerprint,
        patterns: &[("beacon.riskified.com", 6), ("riskified", 4)],
    },
    AntibotSignature {
        vendor: "Signifyd",
        slug: "signifyd",
        kind: AntibotKind::Fingerprint,
        patterns: &[("cdn-scripts.signifyd.com", 6), ("signifyd", 4)],
    },
];
