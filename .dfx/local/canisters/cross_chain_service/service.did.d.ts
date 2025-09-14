import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';
import type { IDL } from '@dfinity/candid';

export interface CrossChainTransaction {
  'id' : string,
  'source_chain' : string,
  'confirmations' : number,
  'status' : string,
  'destination_chain' : string,
  'timestamp' : bigint,
  'tx_hash' : string,
  'block_height' : bigint,
}
export interface DVNAttestation {
  'signature' : Uint8Array | number[],
  'validator' : string,
  'timestamp' : bigint,
  'message_id' : string,
}
export interface DVNMessage {
  'id' : string,
  'source_chain' : number,
  'destination_chain' : number,
  'sender' : string,
  'nonce' : bigint,
  'timestamp' : bigint,
  'payload' : Uint8Array | number[],
}
export type Result = { 'Ok' : string } |
  { 'Err' : string };
export type Result_1 = { 'Ok' : boolean } |
  { 'Err' : string };
export interface _SERVICE {
  'get_dvn_message' : ActorMethod<[string], [] | [DVNMessage]>,
  'get_message_attestations' : ActorMethod<[string], Array<DVNAttestation>>,
  'get_pending_messages' : ActorMethod<[], Array<DVNMessage>>,
  'get_ready_messages' : ActorMethod<[], Array<DVNMessage>>,
  'get_transaction' : ActorMethod<[string], [] | [CrossChainTransaction]>,
  'monitor_evm_transaction' : ActorMethod<[number, string, string], Result>,
  'submit_attestation' : ActorMethod<
    [string, string, Uint8Array | number[]],
    Result
  >,
  'submit_dvn_message' : ActorMethod<
    [number, number, Uint8Array | number[], string],
    string
  >,
  'verify_layerzero_message' : ActorMethod<[number, string, string], Result_1>,
}
export declare const idlFactory: IDL.InterfaceFactory;
export declare const init: (args: { IDL: typeof IDL }) => IDL.Type[];
