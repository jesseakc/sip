import { AuthProvider } from '@/lib/auth-context'
import Shell from './shell'
// Every class used by the shell and its pages lives here. Without this import
// the whole application renders unstyled — no Tailwind utilities and none of the
// SIP design tokens defined in globals.css.
import './globals.css'

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
