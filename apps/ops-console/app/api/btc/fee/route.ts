import { NextResponse } from 'next/server';

export const runtime = 'nodejs';
export const dynamic = 'force-dynamic';

export async function GET() {
  try {
    const res = await fetch('https://blockstream.info/testnet/api/fee-estimates', { cache: 'no-store' });
    const data = await res.json();
    return NextResponse.json(data, { status: res.ok ? 200 : res.status });
  } catch (e: any) {
    return NextResponse.json({ error: e?.message || 'Failed to fetch fee estimates' }, { status: 500 });
  }
}
