#!/usr/bin/env bash
#
# Download monochrome vendor logos from Simple Icons (CC0) into
# assets/logos/{payment,protection}/<slug>.svg.
#
# Vendors not present on Simple Icons get a placeholder shield instead.
# Run once when adding a new signature; commit the SVGs to the repo.

set -euo pipefail

cd "$(dirname "$0")/.."
PAYMENT_DIR="assets/logos/payment"
PROT_DIR="assets/logos/protection"
mkdir -p "$PAYMENT_DIR" "$PROT_DIR"

BASE="https://cdn.simpleicons.org"

# slug   simple-icons-name   destination-dir
ROWS=(
    # ---- Payment processors ----
    "stripe                 stripe              $PAYMENT_DIR"
    "adyen                  adyen               $PAYMENT_DIR"
    "braintree              braintree           $PAYMENT_DIR"
    "checkoutcom            checkout            $PAYMENT_DIR"
    "square                 square              $PAYMENT_DIR"
    "paypal                 paypal              $PAYMENT_DIR"
    "applepay               applepay            $PAYMENT_DIR"
    "googlepay              googlepay           $PAYMENT_DIR"
    "amazonpay              amazonpay           $PAYMENT_DIR"
    "klarna                 klarna              $PAYMENT_DIR"
    "afterpay               afterpay            $PAYMENT_DIR"
    "affirm                 affirm              $PAYMENT_DIR"
    "shopify                shopify             $PAYMENT_DIR"
    "shopifypayments        shopify             $PAYMENT_DIR"
    "bigcommerce            bigcommerce         $PAYMENT_DIR"
    "magento                magento             $PAYMENT_DIR"
    "woocommerce            woocommerce         $PAYMENT_DIR"
    "mollie                 mollie              $PAYMENT_DIR"
    "razorpay               razorpay            $PAYMENT_DIR"
    "salesforcecommercecloud salesforce         $PAYMENT_DIR"
    # ---- Bot/CDN/captcha ----
    "cloudflare             cloudflare          $PROT_DIR"
    "cloudflareturnstile    cloudflare          $PROT_DIR"
    "akamai                 akamai              $PROT_DIR"
    "akamaibotmanager       akamai              $PROT_DIR"
    "awscloudfront          amazonwebservices   $PROT_DIR"
    "awswaf                 amazonwebservices   $PROT_DIR"
    "fastly                 fastly              $PROT_DIR"
    "recaptcha              googlerecaptcha     $PROT_DIR"
    "hcaptcha               hcaptcha            $PROT_DIR"
)

# Vendors below have no Simple Icons coverage; we ship a generic shield SVG
# for them at the bottom of this script.
PLACEHOLDER_SLUGS_PAYMENT=(
    worldpay authorizenet cybersource recurly spreedly
    "2checkout" bambora globalpayments bolt sezzle zip
    gocardless trustly cardinalcommerce gpayments emv3ds
)

PLACEHOLDER_SLUGS_PROT=(
    imperva datadome perimeterx kasada shape
    geetest arkose fingerprintjs sift threatmetrix forter
    riskified signifyd
)

PLACEHOLDER='<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
    <path fill="#888" d="M12 1 3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4z"/>
    <circle cx="12" cy="12" r="2.5" fill="#fff"/>
</svg>'

download_one() {
    local slug="$1" name="$2" dir="$3"
    local out="$dir/$slug.svg"
    if [[ -f "$out" ]]; then
        echo "skip: $out"
        return
    fi
    local url="$BASE/$name"
    if curl -sf -o "$out" "$url"; then
        # Sanity-check we got an SVG, not an HTML 404 page.
        if head -c 5 "$out" | grep -q '<svg\|<?xml'; then
            echo "ok:   $out  ($url)"
        else
            echo "miss: $out  (got non-SVG, replacing with placeholder)"
            echo "$PLACEHOLDER" > "$out"
        fi
    else
        echo "miss: $out  (curl failed, using placeholder)"
        echo "$PLACEHOLDER" > "$out"
    fi
}

for row in "${ROWS[@]}"; do
    # shellcheck disable=SC2086
    set -- $row
    download_one "$1" "$2" "$3"
done

for slug in "${PLACEHOLDER_SLUGS_PAYMENT[@]}"; do
    out="$PAYMENT_DIR/$slug.svg"
    if [[ ! -f "$out" ]]; then
        echo "$PLACEHOLDER" > "$out"
        echo "stub: $out"
    fi
done

for slug in "${PLACEHOLDER_SLUGS_PROT[@]}"; do
    out="$PROT_DIR/$slug.svg"
    if [[ ! -f "$out" ]]; then
        echo "$PLACEHOLDER" > "$out"
        echo "stub: $out"
    fi
done

echo
echo "done. payment:    $(ls "$PAYMENT_DIR" | wc -l) files"
echo "      protection: $(ls "$PROT_DIR" | wc -l) files"
