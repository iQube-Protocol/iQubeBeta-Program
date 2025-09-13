export * from './types/iqube';

export async function getAnchorStatus(iqubeId: string): Promise<AnchorStatus> {
  // TODO: wire to BTC public indexers (Blockstream/Mempool) via packages/indexers
  return { anchored: false, depth: 0, requiredDepth: 6, spvAvailable: false };
}

export async function getDualLockStatus(iqubeId: string): Promise<DualLockStatus> {
  // TODO: compute from EVM + BTC states
  return { evmBound: false, btcAnchored: false };
}

export async function getOrdinalPresence(iqubeId: string): Promise<boolean> {
  // TODO: query ordinal adapter
  return false;
}
