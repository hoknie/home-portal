"use client";

import { useTranslations } from "next-intl";

import { RunTable, useAutomations, useRuns } from "@/entities/automation";
import { useWebhooks } from "@/entities/webhook";
import type { Workflow } from "@/entities/workflow";
import { Sheet, SheetContent, SheetDescription, SheetHeader, SheetTitle, Skeleton } from "@/shared/ui/primitives";

export const MANUAL_PREFIX = "workflow:";

export function useStarterTitle() {
  const t = useTranslations("workflows");
  const automations = useAutomations().data?.data.automations ?? [];
  const webhooks = useWebhooks().data?.data.webhooks ?? [];
  return (id: string) =>
    id.startsWith(MANUAL_PREFIX)
      ? t("byHand")
      : (automations.find((automation) => automation.id === id)?.title ?? webhooks.find((webhook) => webhook.id === id)?.title ?? id);
}

export type WorkflowRunsSheetProps = { workflow: Workflow | null; onClose: () => void; onOpenRun: (id: string) => void };

export function WorkflowRunsSheet({ workflow, onClose, onOpenRun }: WorkflowRunsSheetProps) {
  const t = useTranslations("workflows");
  const runs = useRuns({ workflow: workflow?.id ?? null }, true, workflow !== null);
  const starter = useStarterTitle();
  return (
    <Sheet open={workflow !== null} onOpenChange={(open) => (open ? undefined : onClose())}>
      <SheetContent className="w-full overflow-y-auto sm:max-w-2xl">
        <SheetHeader>
          <SheetTitle>{t("runsTitle", { title: workflow?.title ?? "" })}</SheetTitle>
          <SheetDescription>{t("runsHint")}</SheetDescription>
        </SheetHeader>
        <div className="px-4 pb-6">
          {runs.data ? <RunTable runs={runs.data.runs} titleOf={starter} onOpen={onOpenRun} /> : <Skeleton className="h-40" aria-busy="true" />}
        </div>
      </SheetContent>
    </Sheet>
  );
}
