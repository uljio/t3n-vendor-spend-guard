//! VendorSpendGuard v0.1.0 — enterprise spend policy inside a T3N TEE.
//!
//! Policy / budget / vendor / audit data live in tenant KV maps. No outbound
//! HTTP is required for the core loop, which keeps the agent maintainable and
//! free of unpaid third-party cloud secrets.
#![warn(clippy::style, missing_debug_implementations)]
#![cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]

extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::format;

pub const CONTRACT_VERSION: &str = "0.1.0";

wit_bindgen::generate!({
    world: "vendor-spend-guard",
    path: "wit",
    additional_derives: [
        serde::Deserialize,
        serde::Serialize,
    ],
    generate_all,
});

mod policy;
mod budget;
mod audit;

struct Component;

#[cfg(target_arch = "wasm32")]
impl exports::z::vendor_spend_guard::contracts::Guest for Component {
    fn check_policy(
        req: exports::z::vendor_spend_guard::contracts::GenericInput,
    ) -> Result<Vec<u8>, String> {
        let input = req.input.ok_or("check-policy: missing input")?;
        policy::check_policy(&input)
    }

    fn reserve_budget(
        req: exports::z::vendor_spend_guard::contracts::GenericInput,
    ) -> Result<Vec<u8>, String> {
        let input = req.input.ok_or("reserve-budget: missing input")?;
        budget::reserve_budget(&input)
    }

    fn confirm_purchase(
        req: exports::z::vendor_spend_guard::contracts::GenericInput,
    ) -> Result<Vec<u8>, String> {
        let input = req.input.ok_or("confirm-purchase: missing input")?;
        budget::confirm_purchase(&input)
    }

    fn get_audit(
        req: exports::z::vendor_spend_guard::contracts::GenericInput,
    ) -> Result<Vec<u8>, String> {
        let input = req.input.ok_or("get-audit: missing input")?;
        audit::get_audit(&input)
    }
}

#[cfg(target_arch = "wasm32")]
export!(Component);

/// Build `z:<hex_tid>:<tail>` using tenant-context bytes.
pub fn map_name(tail: &str) -> String {
    #[cfg(target_arch = "wasm32")]
    {
        use crate::host::tenant::tenant_context;
        let tid = tenant_context::tenant_did();
        format!("z:{}:{}", hex::encode(&tid), tail)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        format!("z:native-test:{}", tail)
    }
}

pub fn kv_get(map_tail: &str, key: &str) -> Result<Option<Vec<u8>>, String> {
    #[cfg(target_arch = "wasm32")]
    {
        use crate::host::interfaces::kv_store;
        let name = map_name(map_tail);
        kv_store::get(&name, key.as_bytes()).map_err(|e| format!("kv get {map_tail}/{key}: {e}"))
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (map_tail, key);
        Ok(None)
    }
}

pub fn kv_set(map_tail: &str, key: &str, value: &[u8]) -> Result<(), String> {
    #[cfg(target_arch = "wasm32")]
    {
        use crate::host::interfaces::kv_store;
        let name = map_name(map_tail);
        kv_store::put(&name, key.as_bytes(), value)
            .map_err(|e| format!("kv put {map_tail}/{key}: {e}"))
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (map_tail, key, value);
        Ok(())
    }
}

pub fn log_info(msg: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        use crate::host::interfaces::logging;
        let _ = logging::info(msg);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = msg;
    }
}

#[cfg(test)]
mod tests {
    use super::CONTRACT_VERSION;

    #[test]
    fn contract_version_is_semver() {
        let parts: Vec<&str> = CONTRACT_VERSION.split('.').collect();
        assert_eq!(parts.len(), 3);
        for part in parts {
            assert!(part.parse::<u32>().is_ok());
        }
    }
}
