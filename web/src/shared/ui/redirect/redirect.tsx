"use client";

import { useRouter } from "next/navigation";
import { useTranslations } from "next-intl";
import { useEffect } from "react";

export type RedirectProps = { to: string };

export function Redirect({ to }: RedirectProps) {
  const t = useTranslations("common");
  const router = useRouter();
  useEffect(() => {
    router.replace(`${to}${window.location.search}`);
  }, [router, to]);
  return (
    <p className="p-6 text-sm text-muted-foreground" aria-busy="true">
      {t("redirecting")}
    </p>
  );
}
