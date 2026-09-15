import {
  T3nClient,
  TenantClient,
  setEnvironment,
  loadWasmComponent,
  fetchTrustedManifest,
  eth_get_address,
  metamask_sign,
  createEthAuthInput,
  getNodeUrl,
} from "@terminal3/t3n-sdk";

setEnvironment("testnet");

const T3N_API_KEY = process.env.T3N_API_KEY;
if (!T3N_API_KEY) {
  throw new Error("Set T3N_API_KEY in the environment (never commit it).");
}

const wasmComponent = await loadWasmComponent();
const address = eth_get_address(T3N_API_KEY);

const t3n = new T3nClient({
  trustAnchor: await fetchTrustedManifest("testnet"),
  wasmComponent,
  handlers: {
    EthSign: metamask_sign(address, undefined, T3N_API_KEY),
  },
});

await t3n.handshake();
const did = await t3n.authenticate(createEthAuthInput(address));
const tenantDid = did.value;

console.log("Connected as:", tenantDid);
console.log("Eth address (derived):", address);
console.log("Node URL:", getNodeUrl());

const tenant = new TenantClient({
  t3n,
  baseUrl: getNodeUrl(),
  tenantDid,
});

const me = await tenant.tenant.me();
console.log("TenantClient ready.");
console.log("tenant.me keys:", Object.keys(me || {}));

// Persist non-secret session metadata for later scripts
import { writeFile, mkdir } from "fs/promises";
await mkdir(new URL("../../../.runtime/", import.meta.url), { recursive: true });
await writeFile(
  new URL("../../../.runtime/session.json", import.meta.url),
  JSON.stringify(
    {
      tenantDid,
      ethAddress: address,
      env: "testnet",
      nodeUrl: getNodeUrl(),
      authenticatedAt: new Date().toISOString(),
    },
    null,
    2
  )
);
console.log("Wrote .runtime/session.json (no secrets).");
