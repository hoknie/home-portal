"use client";

import { Puzzle } from "lucide-react";
import { useTranslations } from "next-intl";

export function UnknownWidget({ type, invalid }: { type: string; invalid: boolean }) {
  const t = useTranslations("dashboard");
  return (
    <div className="flex items-center gap-3 rounded-xl border border-dashed p-4 text-sm text-muted-foreground" data-widget-type={type}>
      <Puzzle className="size-5 shrink-0" aria-hidden />
      {invalid ? t("invalidWidget", { type }) : t("unknownWidget", { type })}
    </div>
  );
}
