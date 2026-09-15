use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::format;
use serde::{Deserialize, Serialize};

use crate::{kv_get, kv_set, log_info, audit};

#[derive(Debug, Deserialize)]
struct ReserveReq {
    request_id: String,
    amount_cents: u64,
    #[serde(default = "default_team")]
    team_id: String,
}

fn default_team() -> String { "default".to_string() }

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Budget {
    remaining_cents: u64,
    #[serde(default)]
    daily_spent_cents: u64,
    #[serde(default)]
    currency: String,
}

#[derive(Debug, Serialize)]
struct ReserveOut {
    reserved: bool,
    remaining_cents: u64,
    reservation_id: String,
    reason: String,
}

#[derive(Debug, Deserialize)]
struct ConfirmReq {
    reservation_id: String,
    receipt_ref: String,
    amount_cents: u64,
}

#[derive(Debug, Serialize)]
struct ConfirmOut {
    confirmed: bool,
    audit_seq: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct Reservation {
    reservation_id: String,
    request_id: String,
    amount_cents: u64,
    team_id: String,
    status: String,
}

pub fn reserve_budget(input: &[u8]) -> Result<Vec<u8>, String> {
    let req: ReserveReq = serde_json::from_slice(input)
        .map_err(|e| format!("reserve-budget: bad input: {e}"))?;
    log_info(&format!("reserve-budget id={} amount={}", req.request_id, req.amount_cents));

    let raw = kv_get("budgets", "default")?
        .ok_or("budgets/default missing — seed via tenant SDK")?;
    let mut budget: Budget = serde_json::from_slice(&raw)
        .map_err(|e| format!("budgets/default parse: {e}"))?;

    if req.amount_cents > budget.remaining_cents {
        let out = ReserveOut {
            reserved: false,
            remaining_cents: budget.remaining_cents,
            reservation_id: String::new(),
            reason: format!("insufficient remaining budget {}", budget.remaining_cents),
        };
        return serde_json::to_vec(&out).map_err(|e| e.to_string());
    }

    budget.remaining_cents -= req.amount_cents;
    budget.daily_spent_cents = budget.daily_spent_cents.saturating_add(req.amount_cents);
    kv_set("budgets", "default", &serde_json::to_vec(&budget).map_err(|e| e.to_string())?)?;

    let reservation_id = format!("rsv-{}", req.request_id);
    let reservation = Reservation {
        reservation_id: reservation_id.clone(),
        request_id: req.request_id.clone(),
        amount_cents: req.amount_cents,
        team_id: req.team_id.clone(),
        status: "reserved".to_string(),
    };
    kv_set(
        "budgets",
        &format!("rsv:{}", reservation_id),
        &serde_json::to_vec(&reservation).map_err(|e| e.to_string())?,
    )?;

    audit::append_event("reserve", &format!(
        "{{\"reservation_id\":\"{}\",\"amount_cents\":{},\"remaining\":{}}}",
        reservation_id, req.amount_cents, budget.remaining_cents
    ))?;

    let out = ReserveOut {
        reserved: true,
        remaining_cents: budget.remaining_cents,
        reservation_id,
        reason: "ok".to_string(),
    };
    serde_json::to_vec(&out).map_err(|e| e.to_string())
}

pub fn confirm_purchase(input: &[u8]) -> Result<Vec<u8>, String> {
    let req: ConfirmReq = serde_json::from_slice(input)
        .map_err(|e| format!("confirm-purchase: bad input: {e}"))?;
    log_info(&format!("confirm-purchase rsv={} receipt={}", req.reservation_id, req.receipt_ref));

    let key = format!("rsv:{}", req.reservation_id);
    let raw = kv_get("budgets", &key)?
        .ok_or_else(|| format!("reservation {} not found", req.reservation_id))?;
    let mut reservation: Reservation = serde_json::from_slice(&raw)
        .map_err(|e| format!("reservation parse: {e}"))?;

    if reservation.status != "reserved" {
        return Err(format!("reservation {} status is {}", req.reservation_id, reservation.status));
    }
    if reservation.amount_cents != req.amount_cents {
        return Err("confirm amount mismatch vs reservation".to_string());
    }

    reservation.status = "confirmed".to_string();
    kv_set("budgets", &key, &serde_json::to_vec(&reservation).map_err(|e| e.to_string())?)?;

    let seq = audit::append_event(
        "confirm",
        &format!(
            "{{\"reservation_id\":\"{}\",\"receipt_ref\":\"{}\",\"amount_cents\":{}}}",
            req.reservation_id, req.receipt_ref, req.amount_cents
        ),
    )?;

    let out = ConfirmOut { confirmed: true, audit_seq: seq };
    serde_json::to_vec(&out).map_err(|e| e.to_string())
}
