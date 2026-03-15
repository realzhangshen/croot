// SECURITY: html is generated at build time from local markdown files via
// unified/remark/rehype pipeline. No user-supplied content. Safe to render.
export function DocContent({ html }: { html: string }) {
  return (
    <article
      className="prose prose-lg max-w-3xl"
      dangerouslySetInnerHTML={{ __html: html }}
    />
  );
}
