import fs from 'fs'
import path from 'path'
import { fileURLToPath } from 'url'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const DOCS_DIR = path.join(__dirname, '..', '..', 'docs')
const OUT_FILE = path.join(__dirname, '..', 'public', 'search-data.json')

const SKIP_DIRS = new Set(['.vitepress', 'node_modules', 'public'])

function getMarkdownFiles(dir, base = '') {
  const entries = fs.readdirSync(dir, { withFileTypes: true })
  const files = []
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

function extractTitle(content) {
  const match = content.match(/^#\s+(.+)$/m)
  return match ? match[1] : ''
}

function extractHeadings(content) {
  const headings = []
  for (const match of content.matchAll(/^#{2,4}\s+(.+)$/gm)) {
    headings.push(match[1])
  }
  return headings
}

function stripMarkdown(content) {
  return content
    .replace(/^---[\s\S]*?---\n*/m, '') // frontmatter
    .replace(/```[\s\S]*?```/g, '')     // code blocks
    .replace(/`[^`]+`/g, '')            // inline code
    .replace(/\[([^\]]+)\]\([^)]+\)/g, '$1') // links
    .replace(/[#*_~>|]/g, '')           // markdown syntax
    .replace(/\n{2,}/g, '\n')           // collapse newlines
    .trim()
}

const files = getMarkdownFiles(DOCS_DIR)
const searchData = files.map((file) => {
  const filePath = path.join(DOCS_DIR, file)
  const raw = fs.readFileSync(filePath, 'utf-8')
  const slug = file.replace(/\.md$/, '')

  return {
    slug,
    title: extractTitle(raw) || slug.split('/').pop().replace(/-/g, ' '),
    headings: extractHeadings(raw),
    content: stripMarkdown(raw).slice(0, 500),
  }
})

fs.mkdirSync(path.dirname(OUT_FILE), { recursive: true })
fs.writeFileSync(OUT_FILE, JSON.stringify(searchData, null, 2))
console.log(`Generated search data for ${searchData.length} docs`)
