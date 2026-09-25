"use client";

import { House } from "lucide-react";
import { useTranslations } from "next-intl";

export function Brand() {
  const t = useTranslations("common");
  return (
    <div className="flex items-center gap-2.5 px-3">
      <span className="flex size-8 items-center justify-center rounded-lg bg-primary text-primary-foreground shadow-sm">
        <House className="size-4" aria-hidden />
      </span>
      <span className="font-semibold tracking-tight">{t("appName")}</span>
    </div>
  );
}
