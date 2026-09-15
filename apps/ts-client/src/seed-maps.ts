import { readFile, writeFile } from "fs/promises";
import path from "path";
import { fileURLToPath } from "url";
import { connectTenant } from "./lib/auth.ts";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "../../..");
const contract = JSON.parse(await readFile(path.join(ROOT, ".runtime/contract.json"), "utf8"));
const { tenant, tenantDid } = await connectTenant();
const contractId = contract.contractId;

async function ensureMap(tail: string) {
  try {
    const res = await tenant.maps.create({
      tail,
      visibility: "private",
      writers: { only: [contractId] },
      readers: { only: [contractId] },
    });
    console.log("created map", tail, (res as any)?.map_name || "ok");
    return res;
  } catch (e: any) {
    const msg = String(e?.message || e);
    if (/MapAlreadyExists|already exists/i.test(msg)) {
      console.log("map exists", tail);
      try {
        await tenant.maps.update({
          tail,
          writers: { only: [contractId] },
          readers: { only: [contractId] },
        } as any);
        console.log("updated ACL for", tail);
      } catch (e2: any) {
        console.log("ACL update note for", tail, String(e2?.message || e2).slice(0, 160));
      }
      return null;
    }
    throw e;
  }
}

async function entrySet(tail: string, key: string, value: unknown) {
  const payload = typeof value === "string" ? value : JSON.stringify(value);
  await (tenant as any).executeControl("map-entry-set", {
    map_name: (tenant as any).canonicalName(tail),
    key,
    value: payload,
  });
  console.log("seeded", tail + "/" + key, "bytes", Buffer.byteLength(payload));
}

for (const tail of ["policies", "budgets", "vendors", "audit", "secrets"]) {
  await ensureMap(tail);
}

const policy = JSON.parse(
  await readFile(path.join(ROOT, "policies/example-policy.json"), "utf8")
);
await entrySet("policies", "default", policy);
await entrySet("budgets", "default", {
  remaining_cents: 1000000,
  daily_spent_cents: 0,
  currency: "USD",
});
await entrySet("vendors", "acme-saas", {
  vendor_id: "acme-saas",
  active: true,
  allowed_skus: ["ACM-SEAT-1", "ACM-SEAT-10"],
  reference_price_cents: 40000,
});
await entrySet("vendors", "shadow-corp", {
  vendor_id: "shadow-corp",
  active: false,
  allowed_skus: ["SHD-1"],
  reference_price_cents: 10000,
});
await entrySet("audit", "log", { next_seq: 0, events: [] });
await entrySet("secrets", "payment_gateway_note", "sealed-in-enclave-never-in-llm-context");

await writeFile(
  path.join(ROOT, ".runtime/seeded.json"),
  JSON.stringify(
    {
      tenantDid,
      contractId,
      seededAt: new Date().toISOString(),
      maps: ["policies", "budgets", "vendors", "audit", "secrets"],
    },
    null,
    2
  )
);
console.log("Seeded maps OK");
