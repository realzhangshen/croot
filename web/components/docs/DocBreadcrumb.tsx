export default function DocBreadcrumb({ groupName }: { groupName: string | null }) {
  if (!groupName) return null

  return (
    <div
      style={{
        textTransform: 'uppercase',
        fontSize: 12,
        fontWeight: 600,
        letterSpacing: '0.08em',
        color: 'var(--croot-accent-orange)',
        marginBottom: 8,
      }}
    >
      {groupName}
    </div>
  )
}
