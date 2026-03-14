import Navbar from '@/components/docs/Navbar'
import Sidebar from '@/components/docs/Sidebar'

export default function DocsLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <div className="min-h-screen" style={{ background: 'var(--croot-bg)' }}>
      <Navbar />
      <div className="flex" style={{ maxWidth: 'var(--croot-max-width)', margin: '0 auto' }}>
        <Sidebar />
        <main className="flex-1 min-w-0 px-6 py-8 md:px-12 md:py-12">
          {children}
        </main>
      </div>
    </div>
  )
}
