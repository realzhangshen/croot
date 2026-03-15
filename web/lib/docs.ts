import fs from "fs";
import path from "path";
import { unified } from "unified";
import remarkParse from "remark-parse";
import remarkGfm from "remark-gfm";
import remarkRehype from "remark-rehype";
import rehypeRaw from "rehype-raw";
import rehypeSlug from "rehype-slug";
import rehypeStringify from "rehype-stringify";
import { createHighlighter } from "shiki";

const DOCS_DIR = path.join(process.cwd(), "..", "docs");

let highlighterPromise: ReturnType<typeof createHighlighter> | null = null;

function getHighlighter() {
  if (!highlighterPromise) {
    highlighterPromise = createHighlighter({
      themes: ["github-light", "github-dark"],
      langs: [
        "bash",
        "shell",
        "toml",
        "rust",
        "json",
        "yaml",
        "markdown",
        "typescript",
        "javascript",
      ],
    });
  }
  return highlighterPromise;
}

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

// No user input or shell execution — only reads local markdown files
// from the docs/ directory at build time (static site generation).
export async function getDocBySlug(slug: string): Promise<DocPage> {
  const filePath = path.join(DOCS_DIR, `${slug}.md`);
  const raw = fs.readFileSync(filePath, "utf-8");

  const highlighter = await getHighlighter();

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
    .use(rehypeSlug)
    .use(rehypeStringify)
    .process(raw);

  let html = String(result);

  // Apply syntax highlighting to code blocks
  html = html.replace(
    /<pre><code class="language-(\w+)">([\s\S]*?)<\/code><\/pre>/g,
    (_, lang, code) => {
      const decoded = code
        .replace(/&lt;/g, "<")
        .replace(/&gt;/g, ">")
        .replace(/&amp;/g, "&")
        .replace(/&quot;/g, '"')
        .replace(/&#39;/g, "'");

      try {
        return highlighter.codeToHtml(decoded, {
          lang,
          themes: { light: "github-light", dark: "github-dark" },
          defaultColor: false,
        });
      } catch {
        return `<pre><code>${code}</code></pre>`;
      }
    }
  );

  return { title, html, toc };
}

export function docExists(slug: string): boolean {
  const filePath = path.join(DOCS_DIR, `${slug}.md`);
  return fs.existsSync(filePath);
}
