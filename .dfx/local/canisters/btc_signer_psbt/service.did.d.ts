import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface BitcoinAddress {
  'public_key' : Uint8Array | number[],
  'derivation_path' : Array<Uint8Array | number[]>,
  'address' : string,
}
export interface SignedTransaction {
  'fee' : bigint,
  'size' : number,
  'txid' : string,
  'raw_tx' : string,
}
export interface TransactionInput { 'utxo' : UTXO, 'sequence' : number }
export interface TransactionOutput { 'address' : string, 'amount' : bigint }
export interface UTXO {
  'script_pubkey' : Uint8Array | number[],
  'txid' : string,
  'vout' : number,
  'amount' : bigint,
}
export interface UnsignedTransaction {
  'inputs' : Array<TransactionInput>,
  'locktime' : number,
  'outputs' : Array<TransactionOutput>,
}
export interface _SERVICE {
  'broadcast_transaction' : ActorMethod<
    [string],
    { 'Ok' : string } |
      { 'Err' : string }
  >,
  'create_anchor_transaction' : ActorMethod<
    [string, Array<UTXO>, bigint],
    { 'Ok' : UnsignedTransaction } |
      { 'Err' : string }
  >,
  'get_address_info' : ActorMethod<[string], [] | [BitcoinAddress]>,
  'get_all_addresses' : ActorMethod<[], Array<BitcoinAddress>>,
  'get_btc_address' : ActorMethod<
    [Array<Uint8Array | number[]>],
    { 'Ok' : BitcoinAddress } |
      { 'Err' : string }
  >,
  'get_transaction' : ActorMethod<[string], [] | [SignedTransaction]>,
  'sign_transaction' : ActorMethod<
    [UnsignedTransaction, Array<Uint8Array | number[]>],
    { 'Ok' : SignedTransaction } |
      { 'Err' : string }
  >,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
