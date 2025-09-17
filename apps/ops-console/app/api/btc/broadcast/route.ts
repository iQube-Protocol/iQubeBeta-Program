import { NextResponse } from 'next/server';

export const runtime = 'nodejs';
export const dynamic = 'force-dynamic';

export async function POST(req: Request) {
  try {
    const raw = await req.text();
    if (!raw) {
      return NextResponse.json({ error: 'Missing tx hex in body' }, { status: 400 });
    }
    const res = await fetch('https://blockstream.info/testnet/api/tx', {
      method: 'POST',
      headers: { 'Content-Type': 'text/plain' },
      body: raw,
    });
    const text = await res.text();
    if (!res.ok) {
      return NextResponse.json({ error: 'Upstream error', status: res.status, body: text }, { status: res.status });
    }
    return new NextResponse(text, { status: 200, headers: { 'Content-Type': 'text/plain' } });
  } catch (e: any) {
    return NextResponse.json({ error: e?.message || 'Failed to broadcast' }, { status: 500 });
  }
}
