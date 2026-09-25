"use client";

import { Pencil } from "lucide-react";
import Link from "next/link";
import { useTranslations } from "next-intl";

import { DeleteAutomationButton } from "@/features/delete-automation";
import { RunAutomationButton } from "@/features/run-automation";
import { type Automation, type Catalogue, OutcomeBadge } from "@/entities/automation";
import { routes } from "@/shared/config";
import type { Column } from "@/shared/ui/data-table";
import { Badge, Button } from "@/shared/ui/primitives";
import { RelativeTime } from "@/shared/ui/relative-time";
import { TagList } from "@/shared/ui/tag-list";

import { TriggerText } from "./trigger-text";

export function useAutomationColumns(revision: string | null, catalogue: Catalogue | undefined): Column<Automation>[] {
  const t = useTranslations();
  return [
    {
      key: "title",
      header: t("automations.columns.title"),
      cell: (automation) => (
        <div className="min-w-0">
          <Link href={routes.editAutomation(automation.id)} className="block truncate font-medium hover:underline">
            {automation.title}
          </Link>
          <p className="truncate font-mono text-xs text-muted-foreground">{automation.id}</p>
          <TagList tags={automation.tags} />
        </div>
      ),
    },
    {
      key: "trigger",
      header: t("automations.columns.trigger"),
      cell: (automation) => <TriggerText automation={automation} catalogue={catalogue} />,
    },
    {
      key: "script",
      header: t("automations.columns.script"),
      hideBelow: "lg",
      cell: (automation) => <span className="font-mono text-xs">{automation.run.script}</span>,
    },
    {
      key: "enabled",
      header: t("automations.columns.enabled"),
      hideBelow: "md",
      cell: (automation) => (
        <Badge variant={automation.enabled ? "outline" : "secondary"}>{t(automation.enabled ? "automations.enabled" : "automations.disabled")}</Badge>
      ),
    },
    {
      key: "last",
      header: t("automations.columns.lastRun"),
      hideBelow: "sm",
      cell: (automation) =>
        automation.last_run ? (
          <div className="grid justify-items-start gap-1">
            <OutcomeBadge outcome={automation.last_run.outcome.result} />
            <span className="flex gap-2 text-xs text-muted-foreground">
              <RelativeTime moment={automation.last_run.outcome.last_at} />
              <span>{t("common.milliseconds", { value: automation.last_run.outcome.duration_milliseconds })}</span>
            </span>
          </div>
        ) : (
          <span className="text-xs text-muted-foreground">{t("automations.neverRan")}</span>
        ),
    },
    {
      key: "actions",
      header: t("automations.columns.actions"),
      align: "end",
      cell: (automation) => (
        <div className="flex justify-end gap-1">
          <RunAutomationButton automation={automation} />
          <Button asChild variant="ghost" size="icon">
            <Link href={routes.editAutomation(automation.id)} aria-label={t("common.edit")}>
              <Pencil aria-hidden />
            </Link>
          </Button>
          <DeleteAutomationButton automation={automation} revision={revision} />
        </div>
      ),
    },
  ];
}
