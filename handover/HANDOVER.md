# Handover — VendorSpendGuard → Terminal 3

**Preference:** Prefer Terminal 3 to **host/maintain** after the challenge. Builder (`jio-unlimited` / unlimitedjio@gmail.com) is available as a maintainer under the startup/listing program if invited; otherwise full handover.

## What you get
| Artifact | Location |
|----------|----------|
| TEE contract (Rust → wasm32-wasip2) | `contracts/vendor-spend-guard/` |
| Compiled WASM (rebuild with cargo) | `contracts/vendor-spend-guard/target/wasm32-wasip2/release/vendor_spend_guard.wasm` |
| TS client (auth, register, seed, demo) | `apps/ts-client/` |
| Example policy | `policies/example-policy.json` |
| Live testnet metadata (no secrets) | `.runtime/session.json`, `contract.json`, `agent.json`, `demo-results.json` |
| Bugs / docs friction | `docs/bugs.md` |

## Live identities (testnet)
- **Tenant DID:** `did:t3n:9e635209846abc76b1b5532f31fd8cd124c070fb`
- **Contract:** `z:9e635209846abc76b1b5532f31fd8cd124c070fb:spend-guard` @ `0.1.0` (contract_id `1027`)
- **Org DID:** `did:t3n:ec1303675a4c65f6349a37428cbe6450f7d2bc99`
- **Agent DID:** `did:t3n:e77350acdd1287dfe88171588c34d0ebdc36f86d` (key id `13d06954fcb5229e`)

## Env map (rotate on handover)
| Var | Who | Notes |
|-----|-----|-------|
| `T3N_API_KEY` | Tenant / org admin | Hex secp256k1 from claim page — **never commit** |
| `T3N_ENV` | all | `testnet` (or `sandbox` alias) |
| `AGENT_API_KEY` | Org agent | `t3n_key_…` printed once at `agent create` — needs **own** credits |
| `CONTRACT_VERSION` | register script | bump on re-register |

## Runbook (10 minutes)
```bash
export T3N_API_KEY=0x...   # rotated key
export T3N_ENV=testnet
cd apps/ts-client && npm ci
npx tsx src/quickstart.ts
# optional re-register / re-seed / demo:
npx tsx src/register-contract.ts
npx tsx src/seed-maps.ts
npx tsx src/demo-purchase.ts
```

Rebuild contract:
```bash
source "$HOME/.cargo/env"
cd contracts/vendor-spend-guard
rustup target add wasm32-wasip2
cargo build --target wasm32-wasip2 --release
```

## Cutover checklist
1. Transfer GitHub (or mirror) to Terminal 3 org.
2. Rotate `T3N_API_KEY` / agent keys; re-seed `secrets` map.
3. Confirm org ownership of agent card + contract versions.
4. Re-run `demo-purchase.ts`; archive `.runtime/demo-results.json`.
5. Contact for questions: listing owner `@wardumb` (extra tokens) / builder email above.
