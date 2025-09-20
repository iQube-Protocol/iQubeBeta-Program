import { Actor, HttpAgent } from '@dfinity/agent';

const CANISTER_ID = process.env.NEXT_PUBLIC_SOLANA_SIGNER_CANISTER_ID || process.env.SOLANA_SIGNER_CANISTER_ID;
const HOST = process.env.NEXT_PUBLIC_ICP_HOST || 'http://127.0.0.1:4943';

let agent: HttpAgent | null = null;
let solActor: any = null;

const solanaIDL = ({ IDL }: any) => {
  const TxStatus = IDL.Record({ slot: IDL.Nat64, signature: IDL.Text });
  return IDL.Service({
    get_solana_address: IDL.Func([], [IDL.Text], []),
    get_balance: IDL.Func([IDL.Text], [IDL.Nat64], []),
    request_airdrop: IDL.Func([IDL.Text, IDL.Nat64], [IDL.Text], []),
    build_and_send_transfer: IDL.Func([IDL.Text, IDL.Nat64], [IDL.Text], []),
    get_transaction: IDL.Func([IDL.Text], [IDL.Opt(TxStatus)], []),
    get_latest_blockhash: IDL.Func([], [IDL.Text], []),
    send_raw_transaction: IDL.Func([IDL.Text], [IDL.Text], []),
  });
};

async function getAgent() {
  if (!agent) {
    agent = new HttpAgent({ host: HOST, verifyQuerySignatures: false });
    try { await agent.fetchRootKey(); } catch {}
  }
  return agent;
}

async function getSolActor() {
  if (!solActor) {
    if (!CANISTER_ID) throw new Error('SOLANA_SIGNER_CANISTER_ID not configured');
    const ag = await getAgent();
    solActor = Actor.createActor(solanaIDL, { agent: ag, canisterId: CANISTER_ID });
  }
  return solActor;
}

export async function getSolAddress(): Promise<string> {
  const a = await getSolActor();
  return await a.get_solana_address();
}

export async function getSolBalance(address: string): Promise<bigint> {
  const a = await getSolActor();
  return a.get_balance(address);
}

export async function requestAirdrop(address: string, lamports: bigint): Promise<string> {
  const a = await getSolActor();
  return a.request_airdrop(address, lamports);
}

export async function transferSol(to: string, lamports: bigint): Promise<string> {
  const a = await getSolActor();
  return a.build_and_send_transfer(to, lamports);
}

export async function getSolTx(signature: string): Promise<{ slot: bigint; signature: string } | null> {
  const a = await getSolActor();
  const opt = await a.get_transaction(signature);
  return opt[0] ?? null;
}

export async function getLatestBlockhash(): Promise<string> {
  const a = await getSolActor();
  return a.get_latest_blockhash();
}

export async function sendRawTxBase64(base64: string): Promise<string> {
  const a = await getSolActor();
  return a.send_raw_transaction(base64);
}
