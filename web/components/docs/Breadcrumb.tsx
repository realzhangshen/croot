export function Breadcrumb({ slug }: { slug: string }) {
  const parts = slug.split("/");
  const section = parts[0]?.replace(/-/g, " ") ?? "Docs";

  return (
    <div className="mb-5">
      <p className="text-[12px] font-medium tracking-[0.02em] text-text-muted capitalize">
        {section}
      </p>
    </div>
  );
}
