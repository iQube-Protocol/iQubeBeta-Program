export type IQubeType = 'DataQube' | 'ContentQube' | 'ToolQube' | 'ModelQube' | 'AigentQube';
export type InstanceType = 'template' | 'instance';
export type BusinessModel = 'Buy' | 'Sell' | 'Rent' | 'Lease' | 'Subscribe' | 'Stake' | 'License' | 'Donate';

export interface IQubeTemplate {
  id: string;
  name: string;
  description: string;
  createdAt: string;
  iQubeType?: IQubeType;
  iQubeInstanceType?: InstanceType;
  businessModel?: BusinessModel;
  price?: number;
  version?: string;
  provenance?: number;
  parentTemplateId?: string;
  blakqubeLabels?: any;
  metaExtras?: Array<{ k: string; v: string }>;
  sensitivityScore?: number;
  accuracyScore: number;
  verifiabilityScore: number;
  riskScore: number;
}

/** SDK extensions for UI badges and cross-chain state */
export interface AnchorStatus {
  anchored: boolean;
  depth: number;
  requiredDepth: number;
  txid?: string;
  spvAvailable: boolean;
}

export interface DualLockStatus {
  evmBound: boolean;
  btcAnchored: boolean;
}

export interface ChainAddresses {
  evm?: { chainId: number; contract?: string };
  icp?: { canisterId?: string };
  btc?: { network: 'mainnet' | 'testnet' | 'signet'; ordinalId?: string };
}
