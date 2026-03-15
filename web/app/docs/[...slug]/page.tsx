import { notFound } from "next/navigation";
import { getDocBySlug, docExists } from "@/lib/docs";
import { getAllDocSlugs } from "@/lib/sidebar";
import { DocContent } from "@/components/docs/DocContent";
import { TableOfContents } from "@/components/docs/TableOfContents";
import { Breadcrumb } from "@/components/docs/Breadcrumb";

export function generateStaticParams() {
  return getAllDocSlugs().map((slug) => ({ slug }));
}

export async function generateMetadata({
  params,
}: {
  params: Promise<{ slug: string[] }>;
}) {
  const { slug } = await params;
  const slugStr = slug.join("/");
  if (!docExists(slugStr)) return {};
  const doc = await getDocBySlug(slugStr);
  return { title: `${doc.title} — croot docs` };
}

export default async function DocPage({
  params,
}: {
  params: Promise<{ slug: string[] }>;
}) {
  const { slug } = await params;
  const slugStr = slug.join("/");

  if (!docExists(slugStr)) {
    notFound();
  }

  const doc = await getDocBySlug(slugStr);

  return (
    <div className="flex min-w-0 gap-0">
      <div className="min-w-0 flex-1 py-10 lg:py-12">
        <div className="mx-auto max-w-[760px]">
          <Breadcrumb slug={slugStr} />
          <DocContent html={doc.html} />
        </div>
      </div>
      <TableOfContents entries={doc.toc} />
    </div>
  );
}
