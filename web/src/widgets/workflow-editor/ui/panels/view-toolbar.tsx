"use client";

import { cn } from "cn";
import { CircleHelp, History, Pencil, Play } from "lucide-react";
import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import { Allowed } from "@/entities/session";
import { routes } from "@/shared/config";
import { AddressLink } from "@/shared/ui/address-link";
import { Button, buttonVariants, Panel } from "@/shared/ui/kit";

export type ViewToolbarProps = {
  workflowId: string;
  historyOpen: boolean;
  running: boolean;
  runDisabled: boolean;
  onRun: () => void;
  onLegend: () => void;
  onHistory: () => void;
  remove: ReactNode;
};

export function ViewToolbar({ workflowId, historyOpen, running, runDisabled, onRun, onLegend, onHistory, remove }: ViewToolbarProps) {
  const t = useTranslations("workflowEditor.toolbar");
  return (
    <Panel as="div" padding="none" className="flex flex-wrap items-center gap-1 rounded-xl p-1.5" role="toolbar" aria-label={t("pageLabel")}>
      <Allowed area="workflows" action="execute">
        <Button type="button" size="sm" disabled={running || runDisabled} onClick={onRun}>
          <Play aria-hidden />
          {t("run")}
        </Button>
      </Allowed>
      <AddressLink
        href={historyOpen ? routes.workflow(workflowId) : routes.workflowHistory(workflowId)}
        aria-current={historyOpen ? "page" : undefined}
        onClick={onHistory}
        className={cn(buttonVariants({ variant: historyOpen ? "secondary" : "ghost", size: "sm" }))}
      >
        <History aria-hidden />
        {t("history")}
      </AddressLink>
      <Allowed area="workflows" action="update">
        <AddressLink href={routes.workflowEdit(workflowId)} className={cn(buttonVariants({ variant: "ghost", size: "sm" }))}>
          <Pencil aria-hidden />
          {t("edit")}
        </AddressLink>
      </Allowed>
      <Button type="button" variant="ghost" size="icon" aria-label={t("legend")} onClick={onLegend}>
        <CircleHelp aria-hidden />
      </Button>
      <span className="ms-auto">
        <Allowed area="workflows" action="delete">
          {remove}
        </Allowed>
      </span>
    </Panel>
  );
}
