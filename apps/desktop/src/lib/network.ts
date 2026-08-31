/**
 * Network identity helpers. Labels and features come from Rust `list_networks()`.
 */

import type { ChainFamily, NetworkInfo } from "@/bindings";

export const DEFAULT_NETWORK_ID = "solana-mainnet";

export function normalizeNetworkId(value: unknown): string {
  if (typeof value === "string" && value.trim() !== "") {
    if (value.trim() === "devnet") return "solana-devnet";
    if (value.trim() === "mainnet") return "solana-mainnet";
    return value.trim();
  }
  return DEFAULT_NETWORK_ID;
}

export function findNetwork(
  networks: NetworkInfo[],
  id: unknown,
): NetworkInfo | undefined {
  const normalized = normalizeNetworkId(id);
  return networks.find((n) => n.id === normalized);
}

export function enabledNetworks(networks: NetworkInfo[]): NetworkInfo[] {
  return networks.filter((n) => n.enabled);
}

export function isMainnet(info: NetworkInfo | undefined): boolean {
  return Boolean(info && !info.is_testnet);
}

export function canSwap(info: NetworkInfo | undefined): boolean {
  return Boolean(info?.features.swap && !info.is_testnet);
}

export function networkShortLabel(info: NetworkInfo | undefined, id?: unknown): string {
  if (info) {
    return info.is_testnet ? `${info.name}` : info.name;
  }
  return normalizeNetworkId(id);
}

/** Shell subtitle under the brand mark. */
export function productChainLabel(info: NetworkInfo | undefined): string {
  if (!info) return "Wallet";
  switch (info.family) {
    case "solana":
      return "Solana Wallet";
    case "evm":
      return "Ethereum Wallet";
    case "bitcoin":
      return "Bitcoin Wallet";
    case "sui":
      return "Sui Wallet";
    default:
      return "Wallet";
  }
}

export function nativeAssetId(family: ChainFamily | undefined): string {
  switch (family) {
    case "evm":
      return "eth";
    case "bitcoin":
      return "btc";
    default:
      return "sol";
  }
}

export function receiveWarning(info: NetworkInfo | undefined): string {
  const name = info?.name ?? "this network";
  return `Only send ${name} assets to this address.`;
}

export function familyLabel(family: ChainFamily): string {
  switch (family) {
    case "solana":
      return "Solana";
    case "evm":
      return "Ethereum";
    case "bitcoin":
      return "Bitcoin";
    case "sui":
      return "Sui";
    default:
      return family;
  }
}
