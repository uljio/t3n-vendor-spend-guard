# VendorSpendGuard — Enterprise Spend-Policy Agent on T3N

Organization-friendly T3N agent + TEE contract that enforces vendor spend policies **inside the enclave**: SKU allowlists, price bands, budget reserve/confirm, and an audit log. Policy data lives in tenant KV maps (not hard-coded). **No unpaid third-party cloud APIs** required for the core loop.

Built for the [T3N Agent Build Challenge](https://superteam.fun/earn/listing/t3n-agent-build-challenge/) following [ADK Quickstart](https://docs.terminal3.io/developers/adk/get-started/quickstart) + Walkthrough.

## Why this agent (useful + maintainable)
- One TEE contract, four functions (`check-policy`, `reserve-budget`, `confirm-purchase`, `get-audit`)
- Policy/budget/vendor data in KV maps — change rules without recompiling
- Org-owned agent card path (private by default) so Terminal 3 can host it
- Plain Node/`tsx` client (avoids bundler WASM footguns)
- Handover = env vars + map seed JSON + this README

## Live demo results (testnet, 2026-09-15)
| Step | Result |
|------|--------|
| Authenticate | `did:t3n:9e635209846abc76b1b5532f31fd8cd124c070fb` |
| Register WASM | `z:…:spend-guard` v0.1.0 · contract_id **1027** |
| ALLOW $400 SaaS | `decision: ALLOW` → reserve → confirm `audit_seq: 1` |
| DENY $9000 | exceeds `max_single_purchase` + price band |
| DENY inactive vendor | `shadow-corp inactive` |
| Audit | reserve + confirm events returned |

See `.runtime/demo-results.json` and `screenshots/`.

## Repo layout
```
t3n-agent/
├── apps/ts-client/          # Node scripts: auth, register, seed, demo
├── contracts/vendor-spend-guard/   # Rust → wasm32-wasip2
├── contracts/z-tenant-flight-ref/  # upstream reference clone (docs walkthrough)
├── policies/example-policy.json
├── handover/HANDOVER.md
├── docs/bugs.md
└── screenshots/
```

## Setup
```bash
# 1) Claim key + DID (shown once): https://go.terminal3.io/adk-community
export T3N_API_KEY="0x..."   # never commit
export T3N_ENV=testnet

# 2) Client
cd apps/ts-client
npm install
npx tsx src/quickstart.ts

# 3) Contract toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-wasip2
cd ../../contracts/vendor-spend-guard
cargo build --target wasm32-wasip2 --release

# 4) Register + seed + demo
cd ../../apps/ts-client
npx tsx src/register-contract.ts
npx tsx src/seed-maps.ts
npx tsx src/demo-purchase.ts
```

## Org-owned agent
```bash
npx @terminal3/t3n-sdk org create --name "VendorSpendGuard Org" --env testnet --json
npx @terminal3/t3n-sdk agent create --org "$ORG_DID" --name "VendorSpendGuard" --env testnet --json
# Store agent apiKey once. Agent needs its OWN claim-page credits before invoke().
```

## Handover preference
**Prefer Terminal 3 to host/maintain.** See `handover/HANDOVER.md`.

## License
MIT (demo code). T3N SDK / network subject to Terminal 3 terms.
