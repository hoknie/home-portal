"use client";

import { PowerOff } from "lucide-react";
import Link from "next/link";
import { useTranslations } from "next-intl";

import { routes } from "@/shared/config";

export type ModuleOffNoticeProps = { name: string };

export function ModuleOffNotice({ name }: ModuleOffNoticeProps) {
  const t = useTranslations("modules");
  return (
    <div role="status" className="flex flex-wrap items-start gap-3 rounded-xl border border-status-degraded/40 bg-status-degraded/10 p-4 text-sm">
      <PowerOff className="mt-0.5 size-4 shrink-0" aria-hidden />
      <span className="min-w-0 flex-1">{t("offNotice", { name })}</span>
      <Link className="font-medium underline underline-offset-4" href={routes.adminModules}>
        {t("offNoticeLink")}
      </Link>
    </div>
  );
}
