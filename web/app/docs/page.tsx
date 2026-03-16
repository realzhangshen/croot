"use client";

import { useRouter } from "next/navigation";
import { useEffect } from "react";

export default function DocsIndex() {
  const router = useRouter();

  useEffect(() => {
    router.replace("/docs/guide/getting-started");
  }, [router]);

  return null;
}
