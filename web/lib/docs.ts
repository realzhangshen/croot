import fs from "fs";
import path from "path";
import { unified } from "unified";
import remarkParse from "remark-parse";
import remarkGfm from "remark-gfm";
import remarkRehype from "remark-rehype";
import rehypeRaw from "rehype-raw";
import rehypeShiki from "@shikijs/rehype";
import rehypeSlug from "rehype-slug";
import rehypeStringify from "rehype-stringify";

const DOCS_DIR = path.join(process.cwd(), "..", "docs");

export interface TocEntry {
  id: string;
  text: string;
  level: number;
}

export interface DocPage {
  title: string;
  html: string;
  toc: TocEntry[];
}

// Cache processed docs in memory (persists across dev requests)
const cache = new Map<string, { mtime: number; doc: DocPage }>();

// Reuse a single processor instance (Shiki init is expensive)
// eslint-disable-next-line @typescript-eslint/no-explicit-any
let processor: any = null;

function getProcessor() {
  if (!processor) {
    processor = unified()
      .use(remarkParse)
      .use(remarkGfm)
      .use(remarkRehype, { allowDangerousHtml: true })
      .use(rehypeRaw)
      .use(rehypeShiki, {
        themes: { light: "github-light", dark: "github-dark" },
        defaultColor: false,
        addLanguageClass: true,
      })
      .use(rehypeSlug)
      .use(rehypeStringify);
  }
  return processor;
}

// Only reads local markdown files from docs/ at build time (static site generation).
export async function getDocBySlug(slug: string): Promise<DocPage> {
  const filePath = path.join(DOCS_DIR, `${slug}.md`);
  const stat = fs.statSync(filePath);
  const cached = cache.get(slug);
  if (cached && cached.mtime === stat.mtimeMs) {
    return cached.doc;
  }

  const raw = fs.readFileSync(filePath, "utf-8");

  // Extract TOC from raw markdown
  const toc: TocEntry[] = [];
  const headingRegex = /^(#{2,3})\s+(.+)$/gm;
  let match;
  while ((match = headingRegex.exec(raw)) !== null) {
    const text = match[2].replace(/`([^`]+)`/g, "$1");
    const id = text
      .toLowerCase()
      .replace(/[^\w\s-]/g, "")
      .replace(/\s+/g, "-");
    toc.push({ id, text, level: match[1].length });
  }

  // Extract title from first heading
  const titleMatch = raw.match(/^#\s+(.+)$/m);
  const title = titleMatch ? titleMatch[1] : slug.split("/").pop() || slug;

  const result = await getProcessor().process(raw);
  const html = String(result);
  const doc: DocPage = { title, html, toc };

  cache.set(slug, { mtime: stat.mtimeMs, doc });
  return doc;
}

export function docExists(slug: string): boolean {
  const filePath = path.join(DOCS_DIR, `${slug}.md`);
  return fs.existsSync(filePath);
}
