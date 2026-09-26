"use client";

import { Square } from "lucide-react";
import { useTranslations } from "next-intl";
import { useState } from "react";
import { toast } from "sonner";

import { type Run, isActive, useStopRun } from "@/entities/automation";
import { ConflictError, RequestError } from "@/shared/api";
import { ConfirmDialog } from "@/shared/ui/confirm-dialog";
import { Button } from "@/shared/ui/primitives";

export const NOT_FOUND = 404;

export type StopRunButtonProps = { run: Run; title?: string; labelled?: boolean };

export function StopRunButton({ run, title, labelled = false }: StopRunButtonProps) {
  const t = useTranslations("automations");
  const stop = useStopRun();
  const [open, setOpen] = useState(false);
  if (!isActive(run)) {
    return null;
  }
  const stopping = run.outcome.reason === "stopping";
  const confirm = async () => {
    try {
      await stop.mutateAsync(run.id);
      toast.success(t("stopRequested", { id: run.id }));
    } catch (error) {
      if (error instanceof ConflictError || (error instanceof RequestError && error.status === NOT_FOUND)) {
        toast.error(t("stopFinished"));
      } else {
        toast.error(t("stopFailed"));
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
        aria-label={labelled ? undefined : t("stop")}
        disabled={stopping}
        title={stopping ? t("reasons.stopping") : undefined}
        onClick={() => setOpen(true)}
      >
        <Square aria-hidden />
        {labelled ? t("stop") : null}
      </Button>
      <ConfirmDialog
        open={open}
        onOpenChange={setOpen}
        title={t("stopTitle", { id: run.id, automation: title ?? run.automation })}
        description={t("stopDescription")}
        confirmLabel={t("stop")}
        pending={stop.isPending}
        onConfirm={confirm}
      />
    </>
  );
}
