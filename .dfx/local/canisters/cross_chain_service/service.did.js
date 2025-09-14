export const idlFactory = ({ IDL }) => {
  const DVNMessage = IDL.Record({
    'id' : IDL.Text,
    'source_chain' : IDL.Nat32,
    'destination_chain' : IDL.Nat32,
    'sender' : IDL.Text,
    'nonce' : IDL.Nat64,
    'timestamp' : IDL.Nat64,
    'payload' : IDL.Vec(IDL.Nat8),
  });
  const DVNAttestation = IDL.Record({
    'signature' : IDL.Vec(IDL.Nat8),
    'validator' : IDL.Text,
    'timestamp' : IDL.Nat64,
    'message_id' : IDL.Text,
  });
  const CrossChainTransaction = IDL.Record({
    'id' : IDL.Text,
    'source_chain' : IDL.Text,
    'confirmations' : IDL.Nat32,
    'status' : IDL.Text,
    'destination_chain' : IDL.Text,
    'timestamp' : IDL.Nat64,
    'tx_hash' : IDL.Text,
    'block_height' : IDL.Nat64,
  });
  const Result = IDL.Variant({ 'Ok' : IDL.Text, 'Err' : IDL.Text });
  const Result_1 = IDL.Variant({ 'Ok' : IDL.Bool, 'Err' : IDL.Text });
  return IDL.Service({
    'get_dvn_message' : IDL.Func([IDL.Text], [IDL.Opt(DVNMessage)], ['query']),
    'get_message_attestations' : IDL.Func(
        [IDL.Text],
        [IDL.Vec(DVNAttestation)],
        ['query'],
      ),
    'get_pending_messages' : IDL.Func([], [IDL.Vec(DVNMessage)], ['query']),
    'get_ready_messages' : IDL.Func([], [IDL.Vec(DVNMessage)], ['query']),
    'get_transaction' : IDL.Func(
        [IDL.Text],
        [IDL.Opt(CrossChainTransaction)],
        ['query'],
      ),
    'monitor_evm_transaction' : IDL.Func(
        [IDL.Nat32, IDL.Text, IDL.Text],
        [Result],
        [],
      ),
    'submit_attestation' : IDL.Func(
        [IDL.Text, IDL.Text, IDL.Vec(IDL.Nat8)],
        [Result],
        [],
      ),
    'submit_dvn_message' : IDL.Func(
        [IDL.Nat32, IDL.Nat32, IDL.Vec(IDL.Nat8), IDL.Text],
        [IDL.Text],
        [],
      ),
    'verify_layerzero_message' : IDL.Func(
        [IDL.Nat32, IDL.Text, IDL.Text],
        [Result_1],
        [],
      ),
  });
};
export const init = ({ IDL }) => { return []; };
