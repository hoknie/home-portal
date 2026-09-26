"use client";

import { History } from "lucide-react";
import { useTranslations } from "next-intl";
import { type ReactNode, useState } from "react";

import { DataTable } from "@/shared/ui/data-table";
import { EmptyState } from "@/shared/ui/empty-state";
import { Button } from "@/shared/ui/primitives";
import { RelativeTime } from "@/shared/ui/relative-time";

import { type Run, isActive, messageKeyOf } from "../model/schema";
import { OutcomeBadge } from "./outcome-badge";
import { RunDetails } from "./run-details";

export type RunTableProps = {
  runs: Run[];
  titleOf: (id: string) => string;
  actionsOf?: (run: Run) => ReactNode;
  onOpen?: (id: string) => void;
};

export function RunTable({ runs, titleOf, actionsOf, onOpen }: RunTableProps) {
  const t = useTranslations();
  const [opened, setOpened] = useState<string | null>(null);
  const open = onOpen ?? setOpened;
  return (
    <>
      <DataTable
        rows={runs}
        rowKey={(run) => run.id}
        empty={<EmptyState icon={History} title={t("automations.journalEmpty")} description={t("automations.journalEmptyHint")} />}
        columns={[
          { key: "when", header: t("automations.columns.when"), cell: (run) => <RelativeTime moment={run.outcome.last_at} /> },
          { key: "automation", header: t("automations.columns.title"), cell: (run) => titleOf(run.automation) },
          {
            key: "event",
            header: t("automations.columns.event"),
            hideBelow: "md",
            cell: (run) => (
              <span className="flex flex-wrap items-center gap-2">
                {t(`automationEvents.${messageKeyOf(run.event)}.title` as Parameters<typeof t>[0])}
                {run.fields["run.manual"] === "true" ? <span className="text-xs text-muted-foreground">{t("automations.manual")}</span> : null}
              </span>
            ),
          },
          {
            key: "outcome",
            header: t("automations.columns.outcome"),
            cell: (run) => (
              <span className="flex flex-wrap items-center gap-2">
                <OutcomeBadge outcome={run.outcome.result} />
                {run.outcome.result === "running" ? (
                  <span className="text-xs text-muted-foreground tabular-nums">
                    {t("automations.runningFor", { seconds: Math.floor(run.outcome.duration_milliseconds / 1000) })}
                  </span>
                ) : null}
                {run.outcome.count > 1 ? <span className="text-xs text-muted-foreground">{t("automations.repeated", { count: run.outcome.count })}</span> : null}
              </span>
            ),
          },
          {
            key: "open",
            header: t("automations.columns.details"),
            align: "end",
            cell: (run) => (
              <span className="flex justify-end gap-1">
                {actionsOf && isActive(run) ? actionsOf(run) : null}
                <Button variant="ghost" size="sm" onClick={() => open(run.id)}>
                  {t("automations.openRun")}
                </Button>
              </span>
            ),
          },
        ]}
      />
      {onOpen ? null : <RunDetails runId={opened} onClose={() => setOpened(null)} actions={actionsOf} />}
    </>
  );
}
