# Bugs / docs friction — VendorSpendGuard on T3N ADK (2026-09-15)

Recorded while following refreshed docs against `@terminal3/t3n-sdk@5.2.0` / testnet `cn-api.sg.testnet.t3n.terminal3.io`.

## B1 — `maps.entrySet({ tail, key, value })` rejects valid tails
**Symptom:** `TenantSdkValidationError: Tenant name tail must match /^[a-zA-Z0-9_-][a-zA-Z0-9_.-]{0,127}$/` when calling `tenant.maps.entrySet({ tail: "policies", key: "default", value: ... })` with a clearly valid tail.
**Workaround that worked:** `tenant.executeControl("map-entry-set", { map_name: tenant.canonicalName("policies"), key, value })` per Seed API key docs.
**Suggest:** Align `maps.entrySet` input schema with `executeControl("map-entry-set")`, or document the working shape on Create Tenant KV Maps + Seed API key pages side-by-side.

## B2 — `maps.update` ACL refresh hits the same tail validation error
**Symptom:** After `MapAlreadyExists`, `tenant.maps.update({ tail, writers, readers })` throws the same tail regex error even for `policies` / `budgets`.
**Impact:** Re-binding map ACLs to a new `contract_id` after re-register is awkward.
**Workaround:** Create maps with correct ACL on first registration; avoid re-register when possible; keep a record of `contract_id` (as register docs already warn).

## B3 — API key shown once / easy to lose
**Symptom:** Claim page (`go.terminal3.io/adk-community` → sandbox success UI) shows the secp256k1 key once; masked by default. No recovery UI.
**Impact:** Lost key = dead credits for that DID unless a new claim is taken (new DID).
**Suggest:** Email recovery hint, or rotate-without-new-DID.

## B4 — Agent credits ≠ tenant credits (footgun)
**Symptom:** Org-owned agent minted successfully (`t3n agent create`) returns `t3n_key_…` once, but agent balance starts at zero. Metered `invoke()` / agent session calls fail with insufficient credit until a **second** claim-page visit for the agent.
**Docs:** Member Delegation mentions this; Quickstart could add a one-liner earlier.
**This submission:** Tenant-path demo (self execute) completed live; independent agent invoke left as documented gap to avoid burning a second DID/key mid-deadline.

## B5 — WASM under bundlers
**Docs already warn** Next.js/Turbopack/Vite may break `@terminal3/t3n-sdk` WASM load. We followed guidance: plain Node + `tsx` only. Suggest shipping an official `serverExternalPackages` Next snippet.

## B6 — Trust anchor callout
`trustAnchor: await fetchTrustedManifest("testnet")` is required (constructor throws). Easy to miss when skimming. Bold callout in Quickstart step 3 is good — keep it prominent.

## B7 — Payroll Agent use-case still “Coming Soon”
Dead end from Use Cases nav; link to B2B procurement / Member Delegation until ready.

## B8 — Community comments on Earn listing
Public comments on the Superteam listing report historical testnet / signup friction (Google-only SSO questions, past SDK/testnet mismatch). Our run on 2026-09-15 with SDK 5.2.0 completed auth → register → seed → invoke successfully on SG testnet.
