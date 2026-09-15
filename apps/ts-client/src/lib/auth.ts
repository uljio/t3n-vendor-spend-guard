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

export async function connectTenant() {
  const T3N_API_KEY = process.env.T3N_API_KEY;
  if (!T3N_API_KEY) throw new Error("Set T3N_API_KEY in the environment");

  setEnvironment(process.env.T3N_ENV || "testnet");
  const wasmComponent = await loadWasmComponent();
  const address = eth_get_address(T3N_API_KEY);
  const trustAnchor = await fetchTrustedManifest("testnet");

  const t3n = new T3nClient({
    trustAnchor,
    wasmComponent,
    handlers: { EthSign: metamask_sign(address, undefined, T3N_API_KEY) },
  });
  await t3n.handshake();
  const did = await t3n.authenticate(createEthAuthInput(address));
  const tenantDid = did.value as string;

  const tenant = new TenantClient({
    t3n,
    baseUrl: getNodeUrl(),
    tenantDid,
  });
  await tenant.tenant.me();

  return { t3n, tenant, tenantDid, address, nodeUrl: getNodeUrl(), trustAnchor, wasmComponent };
}
