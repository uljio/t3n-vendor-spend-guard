use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::format;
use serde::{Deserialize, Serialize};

use crate::{kv_get, kv_set, log_info};

#[derive(Debug, Deserialize)]
struct AuditReq {
    #[serde(default = "default_limit")]
    limit: u32,
}

fn default_limit() -> u32 { 20 }

#[derive(Debug, Serialize, Deserialize, Clone)]
struct AuditEvent {
    seq: u64,
    kind: String,
    payload: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct AuditLog {
    next_seq: u64,
    events: Vec<AuditEvent>,
}

pub fn append_event(kind: &str, payload: &str) -> Result<u64, String> {
    let mut log: AuditLog = match kv_get("audit", "log")? {
        Some(b) => serde_json::from_slice(&b).unwrap_or_default(),
        None => AuditLog::default(),
    };
    let seq = log.next_seq;
    log.events.push(AuditEvent {
        seq,
        kind: kind.to_string(),
        payload: payload.to_string(),
    });
    // Cap retained events to keep map entries small
    if log.events.len() > 200 {
        let drop_n = log.events.len() - 200;
        log.events.drain(0..drop_n);
    }
    log.next_seq = seq + 1;
    kv_set("audit", "log", &serde_json::to_vec(&log).map_err(|e| e.to_string())?)?;
    log_info(&format!("audit seq={seq} kind={kind}"));
    Ok(seq)
}

pub fn get_audit(input: &[u8]) -> Result<Vec<u8>, String> {
    let req: AuditReq = if input.is_empty() {
        AuditReq { limit: 20 }
    } else {
        serde_json::from_slice(input).unwrap_or(AuditReq { limit: 20 })
    };
    let log: AuditLog = match kv_get("audit", "log")? {
        Some(b) => serde_json::from_slice(&b).unwrap_or_default(),
        None => AuditLog::default(),
    };
    let limit = req.limit as usize;
    let start = log.events.len().saturating_sub(limit);
    #[derive(Serialize)]
    struct Out<'a> { events: &'a [AuditEvent], next_seq: u64 }
    let out = Out { events: &log.events[start..], next_seq: log.next_seq };
    serde_json::to_vec(&out).map_err(|e| e.to_string())
}
