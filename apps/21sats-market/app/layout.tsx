import type { Metadata } from 'next'

export const metadata: Metadata = {
  title: '21 Sats Market',
  description: 'iQube marketplace for Bitcoin ordinals and data shards',
}

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  )
}
