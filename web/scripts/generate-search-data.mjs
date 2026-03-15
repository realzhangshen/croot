import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const docsDir = path.join(__dirname, "..", "..", "docs");
const outFile = path.join(__dirname, "..", "public", "search-data.json");

const sections = [
  {
    label: "Guide",
    files: [
      "guide/installation",
      "guide/getting-started",
      "guide/configuration",
      "guide/keybindings",
    ],
  },
  {
    label: "Features",
    files: [
      "features/file-preview",
      "features/fuzzy-search",
      "features/file-operations",
      "features/git-integration",
      "features/mouse-support",
      "features/context-menus",
    ],
  },
  {
    label: "Advanced",
    files: ["advanced/cmux-workflow", "advanced/development"],
  },
];

const entries = [];

for (const section of sections) {
  for (const slug of section.files) {
    const filePath = path.join(docsDir, `${slug}.md`);
    const content = fs.readFileSync(filePath, "utf-8");

    const titleMatch = content.match(/^#\s+(.+)$/m);
    const title = titleMatch ? titleMatch[1] : slug.split("/").pop();

    // Strip markdown syntax for plain text search
    const plainText = content
      .replace(/^#+\s+/gm, "")
      .replace(/```[\s\S]*?```/g, "")
      .replace(/`([^`]+)`/g, "$1")
      .replace(/\[([^\]]+)\]\([^)]+\)/g, "$1")
      .replace(/[*_~]/g, "")
      .replace(/\n{2,}/g, "\n")
      .trim();

    entries.push({
      title,
      slug,
      section: section.label,
      content: plainText.slice(0, 2000),
    });
  }
}

fs.writeFileSync(outFile, JSON.stringify(entries, null, 2));
console.log(`Generated search data: ${entries.length} entries`);
