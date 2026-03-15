import type { Metadata } from 'next'
import { JetBrains_Mono } from 'next/font/google'
import '@/tailwind.css'

const jetbrainsMono = JetBrains_Mono({
  subsets: ['latin'],
  variable: '--font-jetbrains-mono',
  display: 'swap',
})

export const metadata: Metadata = {
  title: 'croot — The VS Code sidebar for your terminal',
  description:
    'Navigate files, preview code, and manage your project — all from the command line.',
  icons: { icon: '/favicon.svg' },
  other: { 'theme-color': '#f7f7f4' },
}

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="en">
      <head>
        <link rel="preconnect" href="https://cursor.com" />
        <link rel="preconnect" href="https://fonts.googleapis.com" />
        <link
          rel="preconnect"
          href="https://fonts.gstatic.com"
          crossOrigin=""
        />
      </head>
      <body
        className={`${jetbrainsMono.variable} antialiased`}
        style={{
          fontFamily:
            'CursorGothic, "CursorGothic Fallback", system-ui, "Helvetica Neue", Helvetica, Arial, sans-serif',
          background: '#f7f7f4',
          color: '#26251e',
        }}
      >
        {children}
      </body>
    </html>
  )
}
