"use client";

import { Play } from "lucide-react";
import { useTranslations } from "next-intl";
import { useState } from "react";
import { toast } from "sonner";

import { type Automation, useRunAutomation } from "@/entities/automation";
import { ConflictError, ThrottledError } from "@/shared/api";
import { ConfirmDialog } from "@/shared/ui/confirm-dialog";
import { Button } from "@/shared/ui/primitives";

export type RunAutomationButtonProps = { automation: Automation; labelled?: boolean; onQueued?: (runId: string) => void };

export function RunAutomationButton({ automation, labelled = false, onQueued }: RunAutomationButtonProps) {
  const t = useTranslations("automations");
  const run = useRunAutomation();
  const [open, setOpen] = useState(false);
  const confirm = async () => {
    try {
      const queued = await run.mutateAsync(automation.id);
      const message = t("runQueued", { title: automation.title });
      if (onQueued) {
        toast.success(message, { action: { label: t("openQueuedRun"), onClick: () => onQueued(queued.run_id) } });
      } else {
        toast.success(message);
      }
    } catch (error) {
      if (error instanceof ThrottledError) {
        toast.error(t("runThrottled", { seconds: error.retryAfterSeconds }));
      } else if (error instanceof ConflictError) {
        toast.error(t("runDisabled"));
      } else {
        toast.error(t("runFailed"));
      }
    }
    setOpen(false);
  };
  return (
    <>
      <Button
        type="button"
        variant={labelled ? "outline" : "ghost"}
        size={labelled ? "sm" : "icon"}
        aria-label={labelled ? undefined : t("runNow")}
        disabled={!automation.enabled}
        title={automation.enabled ? undefined : t("runDisabled")}
        onClick={() => setOpen(true)}
      >
        <Play aria-hidden />
        {labelled ? t("runNow") : null}
      </Button>
      <ConfirmDialog
        open={open}
        onOpenChange={setOpen}
        title={t("runNowTitle", { title: automation.title })}
        description={t("runNowDescription", { script: automation.run.script })}
        confirmLabel={t("runNow")}
        pending={run.isPending}
        onConfirm={confirm}
      />
    </>
  );
}
