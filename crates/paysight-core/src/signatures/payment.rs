use serde::{Deserialize, Serialize};

use super::Pattern;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentCategory {
    PrimaryGateway,
    Wallet,
    BuyNowPayLater,
    BankRedirect,
    ThreeDSecure,
    SiteBuilder,
}

impl PaymentCategory {
    pub fn label(self) -> &'static str {
        match self {
            PaymentCategory::PrimaryGateway => "Primary gateway",
            PaymentCategory::Wallet => "Wallet",
            PaymentCategory::BuyNowPayLater => "Buy-now-pay-later",
            PaymentCategory::BankRedirect => "Bank redirect",
            PaymentCategory::ThreeDSecure => "3-D Secure / card auth",
            PaymentCategory::SiteBuilder => "Commerce platform",
        }
    }

    pub fn slug(self) -> &'static str {
        match self {
            PaymentCategory::PrimaryGateway => "primary_gateway",
            PaymentCategory::Wallet => "wallet",
            PaymentCategory::BuyNowPayLater => "bnpl",
            PaymentCategory::BankRedirect => "bank_redirect",
            PaymentCategory::ThreeDSecure => "three_d_secure",
            PaymentCategory::SiteBuilder => "site_builder",
        }
    }
}

pub struct PaymentSignature {
    pub vendor: &'static str,
    /// Stable identifier used to look up logo assets. Lowercase, no spaces.
    pub slug: &'static str,
    pub category: PaymentCategory,
    pub patterns: &'static [Pattern],
}

pub static PAYMENT_SIGNATURES: &[PaymentSignature] = &[
    PaymentSignature {
        vendor: "Stripe",
        slug: "stripe",
        category: PaymentCategory::PrimaryGateway,
        patterns: &[
            ("js.stripe.com", 6),
            ("api.stripe.com", 5),
            ("m.stripe.network", 5),
            ("m.stripe.com", 4),
            ("checkout.stripe.com", 6),
            ("data-stripe", 3),
            ("stripe.elements", 4),
        ],
    },
    PaymentSignature {
        vendor: "Adyen",
        slug: "adyen",
        category: PaymentCategory::PrimaryGateway,
        patterns: &[
            ("checkoutshopper-live.adyen.com", 6),
            ("checkoutshopper-test.adyen.com", 6),
            ("live.adyen.com", 5),
            ("pay.adyen.com", 5),
            ("checkoutanalytics-live.adyen.com", 5),
            ("adyen.com/hpp", 5),
            ("adyen-checkout", 4),
            ("adyencheckout", 3),
            ("AdyenCheckout(", 5),
            ("AdyenPayments", 4),
            ("hasAdyenPaymentsBlock", 5),
            ("Adyen.encrypt", 5),
            ("adyenIntegration", 4),
        ],
    },
    PaymentSignature {
        vendor: "Braintree",
        slug: "braintree",
        category: PaymentCategory::PrimaryGateway,
        patterns: &[
            ("js.braintreegateway.com", 6),
            ("assets.braintreegateway.com", 6),
            ("api.braintreegateway.com", 5),
            ("braintree-web", 4),
            ("braintree.client.create", 5),
        ],
    },
    PaymentSignature {
        vendor: "Worldpay",
        slug: "worldpay",
        category: PaymentCategory::PrimaryGateway,
        patterns: &[
            ("payments.worldpay.com", 6),
            ("secure.worldpay.com", 6),
            ("online.worldpay.com", 5),
            ("access.worldpay.com", 5),
            ("hpp.worldpay.com", 6),
            ("cdn.worldpay.com", 5),
        ],
    },
    PaymentSignature {
        vendor: "Checkout.com",
        slug: "checkoutcom",
        category: PaymentCategory::PrimaryGateway,
        patterns: &[
            ("cdn.checkout.com", 6),
            ("api.checkout.com", 5),
            ("js.checkout.com", 6),
            ("frames.checkout.com", 6),
            ("Frames.init", 4),
        ],
    },
    PaymentSignature {
        vendor: "Square",
        slug: "square",
        category: PaymentCategory::PrimaryGateway,
        patterns: &[
            ("js.squareup.com", 6),
            ("web.squarecdn.com", 6),
            ("squarecdn.com", 4),
            ("Square.payments(", 5),
        ],
    },
    PaymentSignature {
        vendor: "Authorize.Net",
        slug: "authorizenet",
        category: PaymentCategory::PrimaryGateway,
        patterns: &[
            ("js.authorize.net", 6),
            ("accept.authorize.net", 6),
            ("jstest.authorize.net", 6),
            ("AuthorizeNetIFrame", 4),
        ],
    },
    PaymentSignature {
        vendor: "Cybersource",
        slug: "cybersource",
        category: PaymentCategory::PrimaryGateway,
        patterns: &[
            ("flex.cybersource.com", 6),
            ("secureacceptance.cybersource.com", 6),
        ],
    },
    PaymentSignature {
        vendor: "Mollie",
        slug: "mollie",
        category: PaymentCategory::PrimaryGateway,
        patterns: &[
            ("js.mollie.com", 6),
            ("api.mollie.com", 5),
            ("Mollie(", 5),
        ],
    },
    PaymentSignature {
        vendor: "Razorpay",
        slug: "razorpay",
        category: PaymentCategory::PrimaryGateway,
        patterns: &[
            ("checkout.razorpay.com", 6),
            ("api.razorpay.com", 5),
            ("Razorpay(", 5),
        ],
    },
    PaymentSignature {
        vendor: "Recurly",
        slug: "recurly",
        category: PaymentCategory::PrimaryGateway,
        patterns: &[
            ("js.recurly.com", 6),
            ("api.recurly.com", 5),
            ("recurly.configure", 4),
        ],
    },
    PaymentSignature {
        vendor: "Spreedly",
        slug: "spreedly",
        category: PaymentCategory::PrimaryGateway,
        patterns: &[
            ("core.spreedly.com", 6),
            ("Spreedly.init", 5),
        ],
    },
    PaymentSignature {
        vendor: "2Checkout (Verifone)",
        slug: "2checkout",
        category: PaymentCategory::PrimaryGateway,
        patterns: &[
            ("secure.2co.com", 6),
            ("www.2checkout.com", 5),
            ("api.2checkout.com", 5),
        ],
    },
    PaymentSignature {
        vendor: "Bambora / Worldline",
        slug: "bambora",
        category: PaymentCategory::PrimaryGateway,
        patterns: &[
            ("js.na.bambora.com", 6),
            ("api.na.bambora.com", 5),
        ],
    },
    PaymentSignature {
        vendor: "Global Payments / Realex",
        slug: "globalpayments",
        category: PaymentCategory::PrimaryGateway,
        patterns: &[
            ("api.globalpay.com", 6),
            ("hpp.globalpay.com", 6),
            ("js.globalpay.com", 6),
            ("api.realexpayments.com", 6),
            ("hpp.realexpayments.com", 6),
        ],
    },
    PaymentSignature {
        vendor: "Bolt",
        slug: "bolt",
        category: PaymentCategory::PrimaryGateway,
        patterns: &[
            ("connect.bolt.com", 6),
            ("connect-staging.bolt.com", 6),
        ],
    },
    PaymentSignature {
        vendor: "Shopify Payments",
        slug: "shopifypayments",
        category: PaymentCategory::PrimaryGateway,
        patterns: &[
            ("checkout.shopify.com", 6),
            ("shopify_pay", 4),
            ("shop_pay", 3),
            ("Shopify.Checkout", 4),
        ],
    },
    PaymentSignature {
        vendor: "Klarna",
        slug: "klarna",
        category: PaymentCategory::BuyNowPayLater,
        patterns: &[
            ("js.klarna.com", 6),
            ("cdn.klarna.com", 6),
            ("x.klarnacdn.net", 6),
            ("klarnaservices.com", 4),
            ("klarna-checkout", 4),
            ("Klarna.Payments", 5),
        ],
    },
    PaymentSignature {
        vendor: "Afterpay / Clearpay",
        slug: "afterpay",
        category: PaymentCategory::BuyNowPayLater,
        patterns: &[
            ("js.afterpay.com", 6),
            ("static.afterpay.com", 6),
            ("portal.afterpay.com", 5),
            ("afterpay-static.com", 5),
            ("static-us.afterpay.com", 5),
            ("clearpay.co.uk", 4),
        ],
    },
    PaymentSignature {
        vendor: "Affirm",
        slug: "affirm",
        category: PaymentCategory::BuyNowPayLater,
        patterns: &[
            ("cdn1.affirm.com", 6),
            ("cdn2.affirm.com", 6),
            ("affirm.com/js/v2", 6),
            ("affirm.checkout", 4),
        ],
    },
    PaymentSignature {
        vendor: "Sezzle",
        slug: "sezzle",
        category: PaymentCategory::BuyNowPayLater,
        patterns: &[
            ("widget.sezzle.com", 6),
            ("checkout.sezzle.com", 6),
        ],
    },
    PaymentSignature {
        vendor: "Zip / Quadpay",
        slug: "zip",
        category: PaymentCategory::BuyNowPayLater,
        patterns: &[
            ("static.zipmoney.com.au", 6),
            ("api.zip.co", 5),
            ("static.us.zip.co", 6),
            ("quadpay.com", 4),
        ],
    },
    PaymentSignature {
        vendor: "PayPal",
        slug: "paypal",
        category: PaymentCategory::Wallet,
        patterns: &[
            ("paypal.com/sdk/js", 6),
            ("paypalobjects.com", 5),
            ("www.paypal.com/smart/buttons", 6),
            ("paypal.Buttons(", 5),
        ],
    },
    PaymentSignature {
        vendor: "Apple Pay",
        slug: "applepay",
        category: PaymentCategory::Wallet,
        patterns: &[
            ("ApplePaySession", 5),
            ("apple-pay-button", 4),
            ("apple-pay-merchant-id", 5),
            ("applepay.cdn-apple.com", 6),
        ],
    },
    PaymentSignature {
        vendor: "Google Pay",
        slug: "googlepay",
        category: PaymentCategory::Wallet,
        patterns: &[
            ("pay.google.com/gp/p/js/pay.js", 6),
            ("google.payments.api", 5),
            ("googlepay-button", 4),
        ],
    },
    PaymentSignature {
        vendor: "Amazon Pay",
        slug: "amazonpay",
        category: PaymentCategory::Wallet,
        patterns: &[
            ("static-na.payments-amazon.com", 6),
            ("static-eu.payments-amazon.com", 6),
            ("payments.amazon.com", 4),
            ("amazon.Pay.renderButton", 5),
        ],
    },
    PaymentSignature {
        vendor: "GoCardless",
        slug: "gocardless",
        category: PaymentCategory::BankRedirect,
        patterns: &[
            ("pay.gocardless.com", 6),
            ("pay-sandbox.gocardless.com", 6),
            ("api.gocardless.com", 5),
        ],
    },
    PaymentSignature {
        vendor: "Trustly",
        slug: "trustly",
        category: PaymentCategory::BankRedirect,
        patterns: &[
            ("paywithmybank.com", 6),
            ("trustly.com/api", 5),
            ("Trustly.create", 5),
        ],
    },
    PaymentSignature {
        vendor: "Cardinal Commerce / Songbird 3DS",
        slug: "cardinalcommerce",
        category: PaymentCategory::ThreeDSecure,
        patterns: &[
            ("songbird.cardinalcommerce.com", 6),
            ("songbirdstag.cardinalcommerce.com", 6),
            ("centinel.cardinalcommerce.com", 6),
            ("centinelapi.cardinalcommerce.com", 6),
            ("cardinalcommerce.com", 4),
            ("Cardinal.setup", 5),
            ("Cardinal.continue", 5),
            ("Cardinal.on(", 4),
            ("cca_continue", 4),
            ("CardinalJWT", 5),
        ],
    },
    PaymentSignature {
        vendor: "GPayments ActiveServer 3DS",
        slug: "gpayments",
        category: PaymentCategory::ThreeDSecure,
        patterns: &[
            ("gpayments.com.au", 5),
            ("activeserver", 4),
            ("/3ds-web-adapter", 4),
        ],
    },
    PaymentSignature {
        vendor: "EMV 3-D Secure",
        slug: "emv3ds",
        category: PaymentCategory::ThreeDSecure,
        patterns: &[
            ("threedsserver", 3),
            ("emv3ds", 4),
            ("acs.url", 3),
            ("creq=", 3),
        ],
    },
    PaymentSignature {
        vendor: "Shopify",
        slug: "shopify",
        category: PaymentCategory::SiteBuilder,
        patterns: &[
            ("cdn.shopify.com", 4),
            ("Shopify.theme", 3),
            ("/cdn/shop/", 3),
            ("x-shopify-stage", 5),
        ],
    },
    PaymentSignature {
        vendor: "Salesforce Commerce Cloud",
        slug: "salesforcecommercecloud",
        category: PaymentCategory::SiteBuilder,
        patterns: &[
            ("demandware.static", 5),
            ("demandware.edgesuite.net", 5),
            ("dwsharedstore", 3),
            ("on/demandware.store", 5),
        ],
    },
    PaymentSignature {
        vendor: "Adobe Commerce / Magento",
        slug: "magento",
        category: PaymentCategory::SiteBuilder,
        patterns: &[
            ("Magento_", 4),
            ("/static/version", 2),
            ("mage/cookies", 3),
            ("data-mage-init", 4),
        ],
    },
    PaymentSignature {
        vendor: "BigCommerce",
        slug: "bigcommerce",
        category: PaymentCategory::SiteBuilder,
        patterns: &[
            ("cdn11.bigcommerce.com", 5),
            ("bigcommerce.com/s-", 4),
            ("BCData", 3),
        ],
    },
    PaymentSignature {
        vendor: "WooCommerce",
        slug: "woocommerce",
        category: PaymentCategory::SiteBuilder,
        patterns: &[
            ("woocommerce-", 4),
            ("/wp-content/plugins/woocommerce", 5),
            ("wc_add_to_cart_params", 4),
        ],
    },
];
