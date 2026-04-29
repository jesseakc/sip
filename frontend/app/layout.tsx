import { AuthProvider } from '@/lib/auth-context'
import Shell from './shell'

export const metadata = {
  title: 'SIP - Service Intelligence Platform',
  description: 'AI-first service intelligence platform for physical assets',
}

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="en">
      <body className="antialiased">
        <AuthProvider>
          <Shell>{children}</Shell>
        </AuthProvider>
      </body>
    </html>
  )
}
