import type { Metadata } from 'next'

export const metadata: Metadata = {
  title: 'iQube Ops Console',
  description: 'Operations dashboard for iQube protocol monitoring',
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
