use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;
use serde::{Deserialize, Serialize};

use crate::{kv_get, log_info};

#[derive(Debug, Deserialize)]
struct PurchaseRequest {
    sku: String,
    vendor_id: String,
    category: String,
    amount_cents: u64,
    #[serde(default = "default_currency")]
    currency: String,
    #[serde(default)]
    requester: String,
}

fn default_currency() -> String { "USD".to_string() }

#[derive(Debug, Deserialize)]
struct Policy {
    version: u32,
    currency: String,
    max_single_purchase: u64,
    daily_team_cap: u64,
    require_dual_approve_above: u64,
    allowed_categories: Vec<String>,
    #[serde(default)]
    blocked_merchants: Vec<String>,
    #[serde(default = "default_band")]
    price_band_pct: u64,
}

fn default_band() -> u64 { 15 }

#[derive(Debug, Deserialize)]
struct Vendor {
    vendor_id: String,
    #[serde(default)]
    active: bool,
    #[serde(default)]
    allowed_skus: Vec<String>,
    #[serde(default)]
    reference_price_cents: Option<u64>,
}

#[derive(Debug, Serialize)]
struct Decision {
    decision: String,
    reasons: Vec<String>,
    policy_version: u32,
    amount_cents: u64,
    currency: String,
    dual_approve_required: bool,
}

pub fn check_policy(input: &[u8]) -> Result<Vec<u8>, String> {
    let req: PurchaseRequest = serde_json::from_slice(input)
        .map_err(|e| format!("check-policy: bad input json: {e}"))?;
    log_info(&format!("check-policy sku={} vendor={} amount={}", req.sku, req.vendor_id, req.amount_cents));

    let policy_bytes = kv_get("policies", "default")?
        .ok_or("policies/default missing — seed via tenant SDK")?;
    let policy: Policy = serde_json::from_slice(&policy_bytes)
        .map_err(|e| format!("policies/default parse: {e}"))?;

    let mut reasons: Vec<String> = vec![];
    let mut allow = true;

    if req.currency != policy.currency {
        allow = false;
        reasons.push(format!("currency {} != policy {}", req.currency, policy.currency));
    }

    if req.amount_cents > policy.max_single_purchase {
        allow = false;
        reasons.push(format!(
            "amount {} exceeds max_single_purchase {}",
            req.amount_cents, policy.max_single_purchase
        ));
    }

    if !policy.allowed_categories.iter().any(|c| c == &req.category) {
        allow = false;
        reasons.push(format!("category {} not in allowlist", req.category));
    }

    if policy.blocked_merchants.iter().any(|m| m == &req.vendor_id) {
        allow = false;
        reasons.push(format!("vendor {} is blocked", req.vendor_id));
    }

    // Vendor allowlist (optional map entry)
    if let Some(vbytes) = kv_get("vendors", &req.vendor_id)? {
        let vendor: Vendor = serde_json::from_slice(&vbytes)
            .map_err(|e| format!("vendors/{} parse: {e}", req.vendor_id))?;
        if !vendor.active {
            allow = false;
            reasons.push(format!("vendor {} inactive", req.vendor_id));
        }
        if !vendor.allowed_skus.is_empty() && !vendor.allowed_skus.iter().any(|s| s == &req.sku) {
            allow = false;
            reasons.push(format!("sku {} not allowed for vendor {}", req.sku, req.vendor_id));
        }
        if let Some(ref_price) = vendor.reference_price_cents {
            if ref_price > 0 {
                let band = policy.price_band_pct;
                let max_ok = ref_price.saturating_mul(100 + band) / 100;
                if req.amount_cents > max_ok {
                    allow = false;
                    reasons.push(format!(
                        "amount {} above {}% band of reference {}",
                        req.amount_cents, band, ref_price
                    ));
                }
            }
        }
    } else {
        allow = false;
        reasons.push(format!("vendor {} not in vendors map", req.vendor_id));
    }

    // Soft daily cap hint (hard lock happens in reserve-budget)
    if let Some(bbytes) = kv_get("budgets", "default")? {
        #[derive(Deserialize)]
        struct Budget { remaining_cents: u64, #[serde(default)] daily_spent_cents: u64 }
        if let Ok(b) = serde_json::from_slice::<Budget>(&bbytes) {
            if req.amount_cents > b.remaining_cents {
                allow = false;
                reasons.push(format!(
                    "amount {} exceeds remaining budget {}",
                    req.amount_cents, b.remaining_cents
                ));
            }
            if b.daily_spent_cents.saturating_add(req.amount_cents) > policy.daily_team_cap {
                allow = false;
                reasons.push(format!(
                    "would exceed daily_team_cap {} (spent {})",
                    policy.daily_team_cap, b.daily_spent_cents
                ));
            }
        }
    }

    if allow {
        reasons.push("all policy checks passed".to_string());
    }

    let dual = req.amount_cents > policy.require_dual_approve_above;
    let out = Decision {
        decision: if allow { "ALLOW".to_string() } else { "DENY".to_string() },
        reasons,
        policy_version: policy.version,
        amount_cents: req.amount_cents,
        currency: req.currency,
        dual_approve_required: dual,
    };
    serde_json::to_vec(&out).map_err(|e| e.to_string())
}
