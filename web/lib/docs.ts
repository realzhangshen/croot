import fs from 'fs'
import path from 'path'
import matter from 'gray-matter'
import { unified } from 'unified'
import remarkParse from 'remark-parse'
import remarkGfm from 'remark-gfm'
import remarkRehype from 'remark-rehype'
import rehypeSlug from 'rehype-slug'
import rehypeAutolinkHeadings from 'rehype-autolink-headings'
import rehypeStringify from 'rehype-stringify'
import rehypeShiki from '@shikijs/rehype'

const DOCS_DIR = path.join(process.cwd(), '..', 'docs')

const SKIP_DIRS = new Set(['.vitepress', 'node_modules', 'public'])

function getMarkdownFiles(dir: string, base = ''): string[] {
  const entries = fs.readdirSync(dir, { withFileTypes: true })
  const files: string[] = []

  for (const entry of entries) {
    if (SKIP_DIRS.has(entry.name)) continue
    const rel = path.join(base, entry.name)
    if (entry.isDirectory()) {
      files.push(...getMarkdownFiles(path.join(dir, entry.name), rel))
    } else if (entry.name.endsWith('.md') && entry.name !== 'index.md') {
      files.push(rel)
    }
  }

  return files
}

export function getAllSlugs(): string[][] {
  const files = getMarkdownFiles(DOCS_DIR)
  return files.map((f) => {
    const withoutExt = f.replace(/\.md$/, '')
    return withoutExt.split(path.sep)
  })
}

export interface DocHeading {
  id: string
  text: string
  level: number
}

export async function getDocBySlug(slug: string[]): Promise<{
  title: string
  html: string
  editPath: string
  headings: DocHeading[]
}> {
  const filePath = path.join(DOCS_DIR, ...slug) + '.md'
  const source = fs.readFileSync(filePath, 'utf-8')
  const { data, content } = matter(source)

  const processor = unified()
    .use(remarkParse)
    .use(remarkGfm)
    .use(remarkRehype, { allowDangerousHtml: true })
    .use(rehypeSlug)
    .use(rehypeAutolinkHeadings, {
      behavior: 'append',
      properties: { className: ['heading-anchor'], ariaHidden: 'true', tabIndex: -1 },
      content: { type: 'text', value: '#' },
    })
    .use(rehypeShiki, { theme: 'github-light' })
    .use(rehypeStringify, { allowDangerousHtml: true })

  const result = await processor.process(content)

  const relativePath = slug.join('/') + '.md'
  const title = data.title || slug[slug.length - 1].replace(/-/g, ' ').replace(/\b\w/g, (c: string) => c.toUpperCase())

  const html = String(result)

  // Extract h2/h3 headings from rendered HTML
  const headings: DocHeading[] = []
  const headingRegex = /<h([23])\s+id="([^"]+)"[^>]*>([\s\S]*?)<\/h[23]>/g
  let match
  while ((match = headingRegex.exec(html)) !== null) {
    const text = match[3].replace(/<[^>]+>/g, '').trim()
    headings.push({ id: match[2], text, level: Number(match[1]) })
  }

  return {
    title,
    html,
    editPath: `https://github.com/realzhangshen/croot/edit/main/docs/${relativePath}`,
    headings,
  }
}

export async function getAllDocs(): Promise<
  { slug: string[]; title: string; html: string }[]
> {
  const slugs = getAllSlugs()
  const docs = await Promise.all(
    slugs.map(async (slug) => {
      const doc = await getDocBySlug(slug)
      return { slug, title: doc.title, html: doc.html }
    })
  )
  return docs
}
