import { NextResponse } from 'next/server';

export const runtime = 'nodejs';
export const dynamic = 'force-dynamic';

export async function GET(req: Request) {
  try {
    const { searchParams } = new URL(req.url);
    const txid = searchParams.get('txid');
    if (!txid) {
      return NextResponse.json({ error: 'Missing txid' }, { status: 400 });
    }
    const res = await fetch(`https://blockstream.info/testnet/api/tx/${txid}/hex`, { cache: 'no-store' });
    const text = await res.text();
    if (!res.ok) {
      return NextResponse.json({ error: 'Upstream error', status: res.status, body: text }, { status: res.status });
    }
    return new NextResponse(text, { status: 200, headers: { 'Content-Type': 'text/plain' } });
  } catch (e: any) {
    return NextResponse.json({ error: e?.message || 'Failed to fetch tx hex' }, { status: 500 });
  }
}
