"use client";

import { useEffect, useRef } from "react";

// SECURITY: html is generated at build time from local markdown files via
// unified/remark/rehype pipeline. No user-supplied content. Safe to render.
export function DocContent({ html }: { html: string }) {
  const articleRef = useRef<HTMLElement>(null);

  useEffect(() => {
    const article = articleRef.current;
    if (!article) return;

    // Batch all DOM mutations in a single animation frame to avoid layout thrash
    const frameId = requestAnimationFrame(() => {
      // Wrap <pre> blocks with copy button and language label
      article.querySelectorAll("pre").forEach((pre) => {
        if (pre.parentElement?.classList.contains("code-block-wrapper")) return;

        const wrapper = document.createElement("div");
        wrapper.className = "code-block-wrapper";
        pre.parentNode!.insertBefore(wrapper, pre);
        wrapper.appendChild(pre);

        // Extract language from <code class="language-*"> and add label
        const code = pre.querySelector("code");
        const langClass = code?.className
          .split(/\s+/)
          .find((c) => c.startsWith("language-"));
        if (langClass) {
          const lang = langClass.replace("language-", "");
          const label = document.createElement("span");
          label.className = "lang-label";
          label.textContent = lang;
          wrapper.appendChild(label);
        }

        const btn = document.createElement("button");
        btn.className = "copy-btn";
        btn.textContent = "Copy";
        btn.addEventListener("click", () => {
          const text = (code ?? pre).textContent ?? "";
          navigator.clipboard.writeText(text).then(() => {
            btn.textContent = "Copied!";
            setTimeout(() => {
              btn.textContent = "Copy";
            }, 2000);
          });
        });
        wrapper.appendChild(btn);
      });

      // Make headings clickable anchor links
      article.querySelectorAll("h2, h3, h4").forEach((heading) => {
        const id = heading.id;
        if (!id || heading.querySelector("a.heading-anchor")) return;

        const anchor = document.createElement("a");
        anchor.className = "heading-anchor";
        anchor.href = `#${id}`;

        while (heading.firstChild) {
          anchor.appendChild(heading.firstChild);
        }
        heading.appendChild(anchor);
      });
    });

    return () => cancelAnimationFrame(frameId);
  }, [html]);

  return (
    <article
      ref={articleRef}
      className="prose max-w-[760px]"
      dangerouslySetInnerHTML={{ __html: html }}
    />
  );
}
