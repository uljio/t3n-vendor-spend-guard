import { readFile, writeFile, mkdir } from "fs/promises";
import path from "path";
import { fileURLToPath } from "url";
import { connectTenant } from "./lib/auth.ts";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, "../../..");
const WASM_PATH = path.join(
  ROOT,
  "contracts/vendor-spend-guard/target/wasm32-wasip2/release/vendor_spend_guard.wasm"
);
const CONTRACT_TAIL = "spend-guard";
const CONTRACT_VERSION = process.env.CONTRACT_VERSION || "0.1.0";

const { tenant, tenantDid, nodeUrl } = await connectTenant();
const wasmBytes = await readFile(WASM_PATH);

console.log("Registering", CONTRACT_TAIL, CONTRACT_VERSION, "wasm bytes", wasmBytes.length);

let result;
try {
  result = await tenant.contracts.register({
    tail: CONTRACT_TAIL,
    version: CONTRACT_VERSION,
    wasm: wasmBytes,
  });
} catch (e: any) {
  const msg = String(e?.message || e);
  console.error("register error:", msg);
  if (/version is not higher|already/i.test(msg)) {
    console.log("Retrying with bumped patch version...");
    const bumped = CONTRACT_VERSION.replace(/(\d+)$/, (_, n) => String(Number(n) + 1));
    result = await tenant.contracts.register({
      tail: CONTRACT_TAIL,
      version: bumped,
      wasm: wasmBytes,
    });
    (result as any)._versionUsed = bumped;
  } else {
    throw e;
  }
}

const contractId = (result as any).contract_id ?? (result as any).contractId;
const versionUsed = (result as any)._versionUsed || CONTRACT_VERSION;
const tenantId = tenantDid.slice("did:t3n:".length);
const contractName = `z:${tenantId}:${CONTRACT_TAIL}`;

const meta = {
  tenantDid,
  contractTail: CONTRACT_TAIL,
  contractVersion: versionUsed,
  contractId,
  contractName,
  nodeUrl,
  registeredAt: new Date().toISOString(),
  wasmBytes: wasmBytes.length,
};

await mkdir(path.join(ROOT, ".runtime"), { recursive: true });
await writeFile(path.join(ROOT, ".runtime/contract.json"), JSON.stringify(meta, null, 2));
console.log(JSON.stringify(meta, null, 2));
