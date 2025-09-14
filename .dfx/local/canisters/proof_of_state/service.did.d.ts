import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface MerkleBatch {
  'root' : string,
  'created_at' : bigint,
  'btc_anchor_txid' : [] | [string],
  'btc_block_height' : [] | [bigint],
  'receipts' : Array<Receipt>,
}
export interface Receipt {
  'id' : string,
  'timestamp' : bigint,
  'data_hash' : string,
  'merkle_proof' : Array<string>,
}
export interface _SERVICE {
  'anchor' : ActorMethod<[], string>,
  'batch' : ActorMethod<[], string>,
  'get_batches' : ActorMethod<[], Array<MerkleBatch>>,
  'get_pending_count' : ActorMethod<[], bigint>,
  'get_receipt' : ActorMethod<[string], [] | [Receipt]>,
  'issue_receipt' : ActorMethod<[string], string>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
