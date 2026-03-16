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

// Only reads local markdown files from docs/ at build time (static site generation).
export async function getDocBySlug(slug: string): Promise<DocPage> {
  const filePath = path.join(DOCS_DIR, `${slug}.md`);
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

  const result = await unified()
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
    .use(rehypeStringify)
    .process(raw);

  const html = String(result);

  return { title, html, toc };
}

export function docExists(slug: string): boolean {
  const filePath = path.join(DOCS_DIR, `${slug}.md`);
  return fs.existsSync(filePath);
}
