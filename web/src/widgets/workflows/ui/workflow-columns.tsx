"use client";

import { History, Pencil } from "lucide-react";
import { useTranslations } from "next-intl";

import { Allowed } from "@/entities/session";
import { OutcomeBadge } from "@/entities/automation";
import { DeleteWorkflowButton, type Workflow } from "@/entities/workflow";
import { routes } from "@/shared/config";
import { pushAddress } from "@/shared/lib/navigation";
import { AddressLink } from "@/shared/ui/address-link";
import type { Column } from "@/shared/ui/data-table";
import { Badge, buttonVariants } from "@/shared/ui/primitives";
import { RelativeTime } from "@/shared/ui/relative-time";
import { TagList } from "@/shared/ui/tag-list";

import { RunWorkflowDialog } from "./run-workflow-dialog";

export type ColumnsOptions = { revision: string | null; moduleOff: boolean };

export function useWorkflowColumns({ revision, moduleOff }: ColumnsOptions): Column<Workflow>[] {
  const t = useTranslations();
  return [
    {
      key: "title",
      header: t("workflows.columns.title"),
      cell: (workflow) => (
        <div className="grid min-w-0 gap-1">
          <AddressLink href={routes.workflow(workflow.id)} className="truncate font-medium hover:underline">
            {workflow.title}
          </AddressLink>
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
          <AddressLink href={routes.workflowRun(workflow.id, run.id)} className="grid justify-items-start gap-1">
            <OutcomeBadge outcome={run.outcome.result} />
            {run.outcome.result === "running" ? (
              <span className="text-xs text-muted-foreground tabular-nums">{t("automations.runningFor", { seconds: Math.floor(run.outcome.duration_milliseconds / 1000) })}</span>
            ) : run.outcome.result === "queued" ? null : (
              <span className="flex gap-2 text-xs text-muted-foreground">
                <RelativeTime moment={run.outcome.last_at} />
                <span>{t("common.milliseconds", { value: run.outcome.duration_milliseconds })}</span>
              </span>
            )}
          </AddressLink>
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
          <AddressLink href={routes.workflowHistory(workflow.id)} aria-label={t("workflows.runsOf", { title: workflow.title })} className={buttonVariants({ variant: "ghost", size: "icon" })}>
            <History aria-hidden />
          </AddressLink>
          <Allowed area="workflows" action="execute">
            <RunWorkflowDialog workflow={workflow} moduleOff={moduleOff} onQueued={(runId) => pushAddress(routes.workflowRun(workflow.id, runId))} />
          </Allowed>
          <Allowed area="workflows" action="update">
            <AddressLink href={routes.workflowEdit(workflow.id)} aria-label={t("common.edit")} className={buttonVariants({ variant: "ghost", size: "icon" })}>
              <Pencil aria-hidden />
            </AddressLink>
          </Allowed>
          <Allowed area="workflows" action="delete">
            <DeleteWorkflowButton workflow={workflow} revision={revision} />
          </Allowed>
        </div>
      ),
    },
  ];
}
