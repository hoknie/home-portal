"use client";

import { History, Pencil } from "lucide-react";
import Link from "next/link";
import { useTranslations } from "next-intl";

import { OutcomeBadge } from "@/entities/automation";
import type { Workflow } from "@/entities/workflow";
import { routes } from "@/shared/config";
import type { Column } from "@/shared/ui/data-table";
import { Badge, Button } from "@/shared/ui/primitives";
import { RelativeTime } from "@/shared/ui/relative-time";
import { TagList } from "@/shared/ui/tag-list";

import { DeleteWorkflowButton } from "./delete-workflow-button";
import { RunWorkflowDialog } from "./run-workflow-dialog";

export type ColumnsOptions = { revision: string | null; moduleOff: boolean; onRun: (runId: string) => void; onHistory: (workflow: Workflow) => void };

export function useWorkflowColumns({ revision, moduleOff, onRun, onHistory }: ColumnsOptions): Column<Workflow>[] {
  const t = useTranslations();
  return [
    {
      key: "title",
      header: t("workflows.columns.title"),
      cell: (workflow) => (
        <div className="grid min-w-0 gap-1">
          <Link href={routes.editWorkflow(workflow.id)} className="truncate font-medium hover:underline">
            {workflow.title}
          </Link>
          <code className="w-fit font-mono text-xs text-muted-foreground">{workflow.id}</code>
          <TagList tags={workflow.tags} />
        </div>
      ),
    },
    {
      key: "state",
      header: t("workflows.columns.state"),
      hideBelow: "sm",
      cell: (workflow) => <Badge variant={workflow.enabled ? "outline" : "secondary"}>{t(workflow.enabled ? "workflows.enabled" : "workflows.disabled")}</Badge>,
    },
    {
      key: "inputs",
      header: t("workflows.columns.inputs"),
      hideBelow: "md",
      cell: (workflow) =>
        workflow.inputs.length === 0 ? (
          <span className="text-xs text-muted-foreground">{t("workflows.noInputs")}</span>
        ) : (
          <span className="font-mono text-xs">{workflow.inputs.map((input) => input.name).join(", ")}</span>
        ),
    },
    {
      key: "used",
      header: t("workflows.columns.usedBy"),
      hideBelow: "md",
      cell: (workflow) =>
        workflow.used_by.length === 0 ? (
          <span className="text-xs text-muted-foreground">{t("workflows.notUsed")}</span>
        ) : (
          <ul className="grid gap-0.5 text-xs">
            {workflow.used_by.map((usage) => (
              <li key={`${usage.kind}-${usage.id}`}>
                <span className="text-muted-foreground">{t(`workflows.usage.${usage.kind}` as "workflows.usage.automation")}</span> {usage.title}
              </li>
            ))}
          </ul>
        ),
    },
    {
      key: "last",
      header: t("workflows.columns.lastRun"),
      hideBelow: "sm",
      cell: (workflow) => {
        const run = workflow.active_run ?? workflow.last_run;
        return run ? (
          <button type="button" className="flex items-center gap-2 text-xs text-muted-foreground" onClick={() => onRun(run.id)}>
            <OutcomeBadge outcome={run.outcome.result} />
            <RelativeTime moment={run.started_at} />
          </button>
        ) : (
          <span className="text-xs text-muted-foreground">{t("workflows.neverRan")}</span>
        );
      },
    },
    {
      key: "actions",
      header: t("workflows.columns.actions"),
      align: "end",
      cell: (workflow) => (
        <div className="flex justify-end gap-1">
          <Button type="button" variant="ghost" size="icon" aria-label={t("workflows.runsOf", { title: workflow.title })} onClick={() => onHistory(workflow)}>
            <History aria-hidden />
          </Button>
          <RunWorkflowDialog workflow={workflow} moduleOff={moduleOff} onQueued={onRun} />
          <Button asChild variant="ghost" size="icon">
            <Link href={routes.editWorkflow(workflow.id)} aria-label={t("common.edit")}>
              <Pencil aria-hidden />
            </Link>
          </Button>
          <DeleteWorkflowButton workflow={workflow} revision={revision} />
        </div>
      ),
    },
  ];
}
