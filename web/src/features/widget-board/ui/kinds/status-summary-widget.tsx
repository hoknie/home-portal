"use client";

import { useTranslations } from "next-intl";

import { type ServiceView } from "@/entities/service";
import { StatusDot } from "@/shared/ui/status-badge";

import { countByState } from "../../model/grouping";

const ORDER = ["up", "degraded", "down", "unreadable", "unknown"] as const;

export function StatusSummaryWidget({ services }: { settings: unknown; services: ServiceView[] }) {
  const t = useTranslations();
  const counts = countByState(services);
  return (
    <div className="grid grid-cols-2 gap-3 @sm:grid-cols-3 @3xl:grid-cols-6">
      <div className="glass-panel rounded-xl p-4">
        <p className="text-xs text-muted-foreground">{t("widgets.statusSummary.total")}</p>
        <p className="mt-1 text-2xl font-semibold tabular-nums">{services.length}</p>
      </div>
      {ORDER.map((state) => (
        <div key={state} className="glass-panel rounded-xl p-4" data-state={state}>
          <p className="flex items-center gap-2 text-xs text-muted-foreground">
            <StatusDot state={state} />
            {t(`status.${state}`)}
          </p>
          <p className="mt-1 text-2xl font-semibold tabular-nums">{counts[state]}</p>
        </div>
      ))}
    </div>
  );
}
