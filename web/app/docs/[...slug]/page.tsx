import type { Metadata } from 'next'
import { getAllSlugs, getDocBySlug } from '@/lib/docs'
import { sidebar } from '@/lib/sidebar'
import DocBreadcrumb from '@/components/docs/DocBreadcrumb'
import DocContent from '@/components/docs/DocContent'

export async function generateStaticParams() {
  const slugs = getAllSlugs()
  return slugs.map((slug) => ({ slug }))
}

export async function generateMetadata({
  params,
}: {
  params: Promise<{ slug: string[] }>
}): Promise<Metadata> {
  const { slug } = await params
  const doc = await getDocBySlug(slug)
  return {
    title: `${doc.title} — croot`,
  }
}

export default async function DocPage({
  params,
}: {
  params: Promise<{ slug: string[] }>
}) {
  const { slug } = await params
  const doc = await getDocBySlug(slug)

  const currentPath = `/docs/${slug.join('/')}`
  let groupName: string | null = null
  for (const group of sidebar) {
    if (group.items.some((item) => item.link === currentPath)) {
      groupName = group.text
      break
    }
  }

  return (
    <article>
      <DocBreadcrumb groupName={groupName} />
      <DocContent html={doc.html} editPath={doc.editPath} />
    </article>
  )
}
