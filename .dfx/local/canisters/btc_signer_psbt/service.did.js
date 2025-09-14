export const idlFactory = ({ IDL }) => {
  const UTXO = IDL.Record({
    'script_pubkey' : IDL.Vec(IDL.Nat8),
    'txid' : IDL.Text,
    'vout' : IDL.Nat32,
    'amount' : IDL.Nat64,
  });
  const TransactionInput = IDL.Record({
    'utxo' : UTXO,
    'sequence' : IDL.Nat32,
  });
  const TransactionOutput = IDL.Record({
    'address' : IDL.Text,
    'amount' : IDL.Nat64,
  });
  const UnsignedTransaction = IDL.Record({
    'inputs' : IDL.Vec(TransactionInput),
    'locktime' : IDL.Nat32,
    'outputs' : IDL.Vec(TransactionOutput),
  });
  const BitcoinAddress = IDL.Record({
    'public_key' : IDL.Vec(IDL.Nat8),
    'derivation_path' : IDL.Vec(IDL.Vec(IDL.Nat8)),
    'address' : IDL.Text,
  });
  const SignedTransaction = IDL.Record({
    'fee' : IDL.Nat64,
    'size' : IDL.Nat32,
    'txid' : IDL.Text,
    'raw_tx' : IDL.Text,
  });
  return IDL.Service({
    'broadcast_transaction' : IDL.Func(
        [IDL.Text],
        [IDL.Variant({ 'Ok' : IDL.Text, 'Err' : IDL.Text })],
        [],
      ),
    'create_anchor_transaction' : IDL.Func(
        [IDL.Text, IDL.Vec(UTXO), IDL.Nat64],
        [IDL.Variant({ 'Ok' : UnsignedTransaction, 'Err' : IDL.Text })],
        [],
      ),
    'get_address_info' : IDL.Func(
        [IDL.Text],
        [IDL.Opt(BitcoinAddress)],
        ['query'],
      ),
    'get_all_addresses' : IDL.Func([], [IDL.Vec(BitcoinAddress)], ['query']),
    'get_btc_address' : IDL.Func(
        [IDL.Vec(IDL.Vec(IDL.Nat8))],
        [IDL.Variant({ 'Ok' : BitcoinAddress, 'Err' : IDL.Text })],
        [],
      ),
    'get_transaction' : IDL.Func(
        [IDL.Text],
        [IDL.Opt(SignedTransaction)],
        ['query'],
      ),
    'sign_transaction' : IDL.Func(
        [UnsignedTransaction, IDL.Vec(IDL.Vec(IDL.Nat8))],
        [IDL.Variant({ 'Ok' : SignedTransaction, 'Err' : IDL.Text })],
        [],
      ),
  });
};
export const init = ({ IDL }) => { return []; };
