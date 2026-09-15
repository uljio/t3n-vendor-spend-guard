import { readFile, writeFile } from "fs/promises";
import path from "path";
import { fileURLToPath } from "url";
import { getContractVersion } from "@terminal3/t3n-sdk";
import { connectTenant } from "./lib/auth.ts";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "../../..");
const contract = JSON.parse(await readFile(path.join(ROOT, ".runtime/contract.json"), "utf8"));
const { t3n, tenant, tenantDid, nodeUrl } = await connectTenant();

const TENANT_CONTRACT = contract.contractName as string;
const contractVersion =
  (await getContractVersion(nodeUrl, TENANT_CONTRACT).catch(() => null)) ||
  contract.contractVersion;

console.log("Invoking", TENANT_CONTRACT, "v", contractVersion);

async function call(function_name: string, input: Record<string, unknown>) {
  // Prefer executeAndDecode on base client; fall back to tenant.contracts.execute
  try {
    if (typeof (t3n as any).executeAndDecode === "function") {
      return await (t3n as any).executeAndDecode({
        contract_id: TENANT_CONTRACT,
        contract_version: contractVersion,
        function_name,
        input,
      });
    }
  } catch (e) {
    console.log("executeAndDecode failed, trying tenant.contracts.execute:", String((e as any)?.message || e).slice(0, 200));
  }
  const raw = await tenant.contracts.execute({
    contract_id: TENANT_CONTRACT,
    contract_version: contractVersion,
    function_name,
    input,
  });
  return raw;
}

const results: Record<string, unknown> = {};

// 1) ALLOW path — $400 SaaS seat
results.allow_check = await call("check-policy", {
  sku: "ACM-SEAT-1",
  vendor_id: "acme-saas",
  category: "saas",
  amount_cents: 40000,
  currency: "USD",
  requester: "employee:alice",
});
console.log("ALLOW check:", JSON.stringify(results.allow_check));

results.allow_reserve = await call("reserve-budget", {
  request_id: `demo-allow-${Date.now()}`,
  amount_cents: 40000,
  team_id: "default",
});
console.log("ALLOW reserve:", JSON.stringify(results.allow_reserve));

const reservationId =
  (results.allow_reserve as any)?.reservation_id ||
  (results.allow_reserve as any)?.result?.reservation_id;

if (reservationId) {
  results.allow_confirm = await call("confirm-purchase", {
    reservation_id: reservationId,
    receipt_ref: "rcpt-demo-001",
    amount_cents: 40000,
  });
  console.log("ALLOW confirm:", JSON.stringify(results.allow_confirm));
}

// 2) DENY path — $9,000 exceeds max_single_purchase
results.deny_check = await call("check-policy", {
  sku: "ACM-SEAT-10",
  vendor_id: "acme-saas",
  category: "saas",
  amount_cents: 900000,
  currency: "USD",
  requester: "employee:bob",
});
console.log("DENY check:", JSON.stringify(results.deny_check));

// 3) DENY inactive vendor
results.deny_inactive = await call("check-policy", {
  sku: "SHD-1",
  vendor_id: "shadow-corp",
  category: "saas",
  amount_cents: 1000,
  currency: "USD",
  requester: "employee:carol",
});
console.log("DENY inactive:", JSON.stringify(results.deny_inactive));

results.audit = await call("get-audit", { limit: 10 });
console.log("AUDIT:", JSON.stringify(results.audit));

await writeFile(
  path.join(ROOT, ".runtime/demo-results.json"),
  JSON.stringify({ tenantDid, contract: TENANT_CONTRACT, contractVersion, results, at: new Date().toISOString() }, null, 2)
);
console.log("Wrote .runtime/demo-results.json");
