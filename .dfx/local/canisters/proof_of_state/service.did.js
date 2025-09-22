export const idlFactory = ({ IDL }) => {
  const Receipt = IDL.Record({
    'id' : IDL.Text,
    'timestamp' : IDL.Nat64,
    'data_hash' : IDL.Text,
    'merkle_proof' : IDL.Vec(IDL.Text),
  });
  const MerkleBatch = IDL.Record({
    'root' : IDL.Text,
    'created_at' : IDL.Nat64,
    'btc_anchor_txid' : IDL.Opt(IDL.Text),
    'btc_block_height' : IDL.Opt(IDL.Nat64),
    'receipts' : IDL.Vec(Receipt),
  });
  const BurnState = IDL.Record({
    'receipt_id' : IDL.Text,
    'timestamp' : IDL.Nat64,
    'message_id' : IDL.Text,
    'burned' : IDL.Bool,
  });
  return IDL.Service({
    'anchor' : IDL.Func([], [IDL.Text], []),
    'batch' : IDL.Func([], [IDL.Text], []),
    'get_anchor_status' : IDL.Func([], [IDL.Text], ['query']),
    'get_batches' : IDL.Func([], [IDL.Vec(MerkleBatch)], ['query']),
    'get_burn_state' : IDL.Func([IDL.Text], [IDL.Opt(BurnState)], ['query']),
    'get_last_anchor' : IDL.Func([], [IDL.Opt(IDL.Text)], ['query']),
    'get_pending_count' : IDL.Func([], [IDL.Nat64], ['query']),
    'get_receipt' : IDL.Func([IDL.Text], [IDL.Opt(Receipt)], ['query']),
    'issue_receipt' : IDL.Func([IDL.Text], [IDL.Text], []),
    'set_burn_state' : IDL.Func([IDL.Text, IDL.Text, IDL.Bool], [IDL.Text], []),
  });
};
export const init = ({ IDL }) => { return []; };
