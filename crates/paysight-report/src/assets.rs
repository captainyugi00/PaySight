//! Embedded vendor logos. SVG sources from Simple Icons (CC0) where
//! available; a generic monochrome shield placeholder otherwise.
//!
//! Logos are inlined directly into the rendered HTML — no rasterization,
//! no external requests, no font dependency.

/// Look up a payment vendor's monochrome SVG by its signature slug.
pub fn payment_logo(slug: &str) -> Option<&'static [u8]> {
    Some(match slug {
        "stripe" => include_bytes!("../../../assets/logos/payment/stripe.svg"),
        "adyen" => include_bytes!("../../../assets/logos/payment/adyen.svg"),
        "braintree" => include_bytes!("../../../assets/logos/payment/braintree.svg"),
        "checkoutcom" => include_bytes!("../../../assets/logos/payment/checkoutcom.svg"),
        "square" => include_bytes!("../../../assets/logos/payment/square.svg"),
        "paypal" => include_bytes!("../../../assets/logos/payment/paypal.svg"),
        "applepay" => include_bytes!("../../../assets/logos/payment/applepay.svg"),
        "googlepay" => include_bytes!("../../../assets/logos/payment/googlepay.svg"),
        "amazonpay" => include_bytes!("../../../assets/logos/payment/amazonpay.svg"),
        "klarna" => include_bytes!("../../../assets/logos/payment/klarna.svg"),
        "afterpay" => include_bytes!("../../../assets/logos/payment/afterpay.svg"),
        "affirm" => include_bytes!("../../../assets/logos/payment/affirm.svg"),
        "shopify" => include_bytes!("../../../assets/logos/payment/shopify.svg"),
        "shopifypayments" => {
            include_bytes!("../../../assets/logos/payment/shopifypayments.svg")
        }
        "bigcommerce" => include_bytes!("../../../assets/logos/payment/bigcommerce.svg"),
        "magento" => include_bytes!("../../../assets/logos/payment/magento.svg"),
        "woocommerce" => include_bytes!("../../../assets/logos/payment/woocommerce.svg"),
        "mollie" => include_bytes!("../../../assets/logos/payment/mollie.svg"),
        "razorpay" => include_bytes!("../../../assets/logos/payment/razorpay.svg"),
        "salesforcecommercecloud" => {
            include_bytes!("../../../assets/logos/payment/salesforcecommercecloud.svg")
        }
        "worldpay" => include_bytes!("../../../assets/logos/payment/worldpay.svg"),
        "authorizenet" => include_bytes!("../../../assets/logos/payment/authorizenet.svg"),
        "cybersource" => include_bytes!("../../../assets/logos/payment/cybersource.svg"),
        "recurly" => include_bytes!("../../../assets/logos/payment/recurly.svg"),
        "spreedly" => include_bytes!("../../../assets/logos/payment/spreedly.svg"),
        "2checkout" => include_bytes!("../../../assets/logos/payment/2checkout.svg"),
        "bambora" => include_bytes!("../../../assets/logos/payment/bambora.svg"),
        "globalpayments" => {
            include_bytes!("../../../assets/logos/payment/globalpayments.svg")
        }
        "bolt" => include_bytes!("../../../assets/logos/payment/bolt.svg"),
        "sezzle" => include_bytes!("../../../assets/logos/payment/sezzle.svg"),
        "zip" => include_bytes!("../../../assets/logos/payment/zip.svg"),
        "gocardless" => include_bytes!("../../../assets/logos/payment/gocardless.svg"),
        "trustly" => include_bytes!("../../../assets/logos/payment/trustly.svg"),
        "cardinalcommerce" => {
            include_bytes!("../../../assets/logos/payment/cardinalcommerce.svg")
        }
        "gpayments" => include_bytes!("../../../assets/logos/payment/gpayments.svg"),
        "emv3ds" => include_bytes!("../../../assets/logos/payment/emv3ds.svg"),
        _ => return None,
    })
}

pub fn protection_logo(slug: &str) -> Option<&'static [u8]> {
    Some(match slug {
        "cloudflare" => include_bytes!("../../../assets/logos/protection/cloudflare.svg"),
        "cloudflareturnstile" => {
            include_bytes!("../../../assets/logos/protection/cloudflareturnstile.svg")
        }
        "akamai" => include_bytes!("../../../assets/logos/protection/akamai.svg"),
        "akamaibotmanager" => {
            include_bytes!("../../../assets/logos/protection/akamaibotmanager.svg")
        }
        "awscloudfront" => {
            include_bytes!("../../../assets/logos/protection/awscloudfront.svg")
        }
        "awswaf" => include_bytes!("../../../assets/logos/protection/awswaf.svg"),
        "fastly" => include_bytes!("../../../assets/logos/protection/fastly.svg"),
        "recaptcha" => include_bytes!("../../../assets/logos/protection/recaptcha.svg"),
        "hcaptcha" => include_bytes!("../../../assets/logos/protection/hcaptcha.svg"),
        "imperva" => include_bytes!("../../../assets/logos/protection/imperva.svg"),
        "datadome" => include_bytes!("../../../assets/logos/protection/datadome.svg"),
        "perimeterx" => include_bytes!("../../../assets/logos/protection/perimeterx.svg"),
        "kasada" => include_bytes!("../../../assets/logos/protection/kasada.svg"),
        "shape" => include_bytes!("../../../assets/logos/protection/shape.svg"),
        "geetest" => include_bytes!("../../../assets/logos/protection/geetest.svg"),
        "arkose" => include_bytes!("../../../assets/logos/protection/arkose.svg"),
        "fingerprintjs" => {
            include_bytes!("../../../assets/logos/protection/fingerprintjs.svg")
        }
        "sift" => include_bytes!("../../../assets/logos/protection/sift.svg"),
        "threatmetrix" => {
            include_bytes!("../../../assets/logos/protection/threatmetrix.svg")
        }
        "forter" => include_bytes!("../../../assets/logos/protection/forter.svg"),
        "riskified" => include_bytes!("../../../assets/logos/protection/riskified.svg"),
        "signifyd" => include_bytes!("../../../assets/logos/protection/signifyd.svg"),
        _ => return None,
    })
}

pub const PLACEHOLDER_SVG: &[u8] = br##"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
    <path fill="currentColor" d="M12 1 3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4z" opacity="0.6"/>
    <circle cx="12" cy="12" r="2.5" fill="#fff"/>
</svg>"##;

/// CSS for the HTML report — kept as one big string so the generated file
/// is fully self-contained.
pub const STYLESHEET: &str = include_str!("./report.css");

/// JS for the HTML report (counter animations + intersection observer
/// reveals + filter logic).
pub const SCRIPT: &str = include_str!("./report.js");
