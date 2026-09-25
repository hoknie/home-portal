"use client";

import { AlertTriangle, History } from "lucide-react";
import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import { Skeleton } from "@/shared/ui/primitives";

export type WidgetFrameProps = {
  title: string;
  stale?: boolean;
  problem?: string | null;
  loading?: boolean;
  children: ReactNode;
};

export function WidgetFrame({ title, stale = false, problem = null, loading = false, children }: WidgetFrameProps) {
  const t = useTranslations("widgets");
  return (
    <section className="grid gap-3">
      <div className="flex items-center gap-2">
        <h2 className="text-base font-semibold">{title}</h2>
        {stale ? (
          <span
            data-stale="true"
            title={problem ?? undefined}
            className="inline-flex items-center gap-1 rounded-full border border-status-degraded/40 bg-status-degraded/10 px-2 py-0.5 text-xs text-status-degraded"
          >
            <History className="size-3" aria-hidden />
            {t("stale")}
          </span>
        ) : null}
      </div>
      {loading ? <Skeleton className="h-24 w-full" aria-busy="true" /> : children}
    </section>
  );
}

export function WidgetProblem({ message }: { message: string }) {
  const t = useTranslations("widgets");
  return (
    <div role="alert" className="flex items-start gap-3 rounded-xl border border-destructive/30 bg-destructive/5 p-4 text-sm">
      <AlertTriangle className="mt-0.5 size-4 shrink-0 text-destructive" aria-hidden />
      <div>
        <p className="font-medium">{t("failed")}</p>
        <p className="text-muted-foreground">{message}</p>
      </div>
    </div>
  );
}
