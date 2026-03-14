/**
 * DocContent renders build-time HTML generated from our own trusted markdown
 * files in the docs/ directory. No user-supplied or external content enters
 * the rendering pipeline — the unified/remark/rehype chain only processes
 * local .md files we control. This makes dangerouslySetInnerHTML safe here.
 */
export default function DocContent({
  html,
  editPath,
}: {
  html: string
  editPath: string
}) {
  return (
    <>
      {/* SAFETY: html is generated at build time from trusted local docs/*.md files only */}
      <div
        className="prose-doc"
        dangerouslySetInnerHTML={{ __html: html }}
      />

      <div
        style={{
          marginTop: 48,
          paddingTop: 24,
          borderTop: '1px solid var(--croot-border)',
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
        }}
      >
        <a
          href={editPath}
          target="_blank"
          rel="noopener"
          style={{
            fontSize: '0.85rem',
            color: 'var(--croot-text-muted)',
            textDecoration: 'none',
            transition: 'color var(--croot-dur-fast) var(--croot-ease)',
          }}
        >
          Edit this page on GitHub
        </a>
      </div>
    </>
  )
}
