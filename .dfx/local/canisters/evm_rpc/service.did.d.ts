import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface BlockInfo {
  'hash' : string,
  'number' : bigint,
  'timestamp' : bigint,
  'gas_limit' : bigint,
  'gas_used' : bigint,
  'transaction_count' : number,
  'parent_hash' : string,
}
export interface EVMChainConfig {
  'block_explorer' : string,
  'name' : string,
  'chain_id' : number,
  'rpc_url' : string,
  'native_token' : string,
}
export interface EVMLog {
  'log_index' : number,
  'data' : string,
  'topics' : Array<string>,
  'address' : string,
}
export type Result = { 'Ok' : TransactionReceipt } |
  { 'Err' : string };
export type Result_1 = { 'Ok' : BlockInfo } |
  { 'Err' : string };
export type Result_2 = { 'Ok' : bigint } |
  { 'Err' : string };
export interface TransactionReceipt {
  'status' : boolean,
  'block_hash' : string,
  'transaction_index' : number,
  'logs' : Array<EVMLog>,
  'to_address' : string,
  'block_number' : bigint,
  'from_address' : string,
  'gas_used' : bigint,
  'tx_hash' : string,
}
export interface _SERVICE {
  'get_block_info' : ActorMethod<[number, bigint], Result_1>,
  'get_cached_block' : ActorMethod<[bigint], [] | [BlockInfo]>,
  'get_cached_receipt' : ActorMethod<[string], [] | [TransactionReceipt]>,
  'get_chain_config' : ActorMethod<[number], [] | [EVMChainConfig]>,
  'get_latest_block_number' : ActorMethod<[number], Result_2>,
  'get_supported_chains' : ActorMethod<[], Array<EVMChainConfig>>,
  'get_transaction_receipt' : ActorMethod<[number, string], Result>,
  'init_chain_configs' : ActorMethod<[], undefined>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
