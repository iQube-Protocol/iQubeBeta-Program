export const idlFactory = ({ IDL }) => {
  const BlockInfo = IDL.Record({
    'hash' : IDL.Text,
    'number' : IDL.Nat64,
    'timestamp' : IDL.Nat64,
    'gas_limit' : IDL.Nat64,
    'gas_used' : IDL.Nat64,
    'transaction_count' : IDL.Nat32,
    'parent_hash' : IDL.Text,
  });
  const Result_1 = IDL.Variant({ 'Ok' : BlockInfo, 'Err' : IDL.Text });
  const EVMLog = IDL.Record({
    'log_index' : IDL.Nat32,
    'data' : IDL.Text,
    'topics' : IDL.Vec(IDL.Text),
    'address' : IDL.Text,
  });
  const TransactionReceipt = IDL.Record({
    'status' : IDL.Bool,
    'block_hash' : IDL.Text,
    'transaction_index' : IDL.Nat32,
    'logs' : IDL.Vec(EVMLog),
    'to_address' : IDL.Text,
    'block_number' : IDL.Nat64,
    'from_address' : IDL.Text,
    'gas_used' : IDL.Nat64,
    'tx_hash' : IDL.Text,
  });
  const EVMChainConfig = IDL.Record({
    'block_explorer' : IDL.Text,
    'name' : IDL.Text,
    'chain_id' : IDL.Nat32,
    'rpc_url' : IDL.Text,
    'native_token' : IDL.Text,
  });
  const Result_2 = IDL.Variant({ 'Ok' : IDL.Nat64, 'Err' : IDL.Text });
  const Result = IDL.Variant({ 'Ok' : TransactionReceipt, 'Err' : IDL.Text });
  return IDL.Service({
    'get_block_info' : IDL.Func([IDL.Nat32, IDL.Nat64], [Result_1], []),
    'get_cached_block' : IDL.Func([IDL.Nat64], [IDL.Opt(BlockInfo)], ['query']),
    'get_cached_receipt' : IDL.Func(
        [IDL.Text],
        [IDL.Opt(TransactionReceipt)],
        ['query'],
      ),
    'get_chain_config' : IDL.Func(
        [IDL.Nat32],
        [IDL.Opt(EVMChainConfig)],
        ['query'],
      ),
    'get_latest_block_number' : IDL.Func([IDL.Nat32], [Result_2], []),
    'get_supported_chains' : IDL.Func([], [IDL.Vec(EVMChainConfig)], ['query']),
    'get_transaction_receipt' : IDL.Func([IDL.Nat32, IDL.Text], [Result], []),
    'init_chain_configs' : IDL.Func([], [], []),
  });
};
export const init = ({ IDL }) => { return []; };
