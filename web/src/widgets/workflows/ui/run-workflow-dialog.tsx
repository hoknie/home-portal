"use client";

import { Play } from "lucide-react";
import { useTranslations } from "next-intl";
import { useState } from "react";
import { toast } from "sonner";

import { InputsForm, type Workflow, initialValues, runValues, useRunWorkflow } from "@/entities/workflow";
import { ConflictError, ThrottledError } from "@/shared/api";
import { Button, Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/shared/ui/kit";

export type RunWorkflowDialogProps = { workflow: Workflow; moduleOff: boolean; onQueued: (runId: string) => void };

export function RunWorkflowDialog({ workflow, moduleOff, onQueued }: RunWorkflowDialogProps) {
  const t = useTranslations();
  const run = useRunWorkflow();
  const [open, setOpen] = useState(false);
  const [inputs, setInputs] = useState<Record<string, unknown>>(() => initialValues(workflow.inputs));
  const confirm = async () => {
    try {
      const queued = await run.mutateAsync({ id: workflow.id, inputs: runValues(workflow.inputs, inputs) });
      setOpen(false);
      onQueued(queued.run_id);
    } catch (error) {
      if (error instanceof ThrottledError) {
        toast.error(t("workflows.runThrottled", { seconds: error.retryAfterSeconds }));
      } else if (error instanceof ConflictError) {
        toast.error(t("workflows.runRefused"));
      } else {
        toast.error(t("workflows.runFailed"));
      }
    }
  };
  const reason = moduleOff ? t("modules.offReason") : workflow.enabled ? undefined : t("workflows.runDisabled");
  return (
    <>
      <span title={reason}>
        <Button type="button" variant="ghost" size="icon" aria-label={t("workflows.runNow")} disabled={reason !== undefined} onClick={() => setOpen(true)}>
          <Play aria-hidden />
        </Button>
      </span>
      <Dialog open={open} onOpenChange={setOpen}>
        <DialogContent closeLabel={t("common.close")}>
          <DialogHeader>
            <DialogTitle>{t("workflows.runNowTitle", { title: workflow.title })}</DialogTitle>
            <DialogDescription>{t("workflows.runNowDescription")}</DialogDescription>
          </DialogHeader>
          <InputsForm idPrefix="run-input" inputs={workflow.inputs} values={inputs} onChange={setInputs} />
          <DialogFooter>
            <Button variant="outline" onClick={() => setOpen(false)} disabled={run.isPending}>
              {t("common.cancel")}
            </Button>
            <Button onClick={() => void confirm()} disabled={run.isPending}>
              <Play aria-hidden />
              {t("workflows.runNow")}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}
