"use client";

import { Trash2 } from "lucide-react";
import { useTranslations } from "next-intl";
import { useState } from "react";
import { toast } from "sonner";

import { type Automation, useDeleteAutomation } from "@/entities/automation";
import { ConflictError } from "@/shared/api";
import { ConfirmDialog } from "@/shared/ui/confirm-dialog";
import { Button } from "@/shared/ui/primitives";

export type DeleteAutomationButtonProps = { automation: Automation; revision: string | null };

export function DeleteAutomationButton({ automation, revision }: DeleteAutomationButtonProps) {
  const t = useTranslations();
  const remove = useDeleteAutomation();
  const [open, setOpen] = useState(false);
  const confirm = async () => {
    try {
      await remove.mutateAsync({ id: automation.id, revision });
      toast.success(t("automations.deleted"));
    } catch (error) {
      toast.error(t(error instanceof ConflictError ? "errors.conflict" : "errors.generic"));
    }
    setOpen(false);
  };
  return (
    <>
      <Button variant="ghost" size="icon" aria-label={t("common.delete")} onClick={() => setOpen(true)}>
        <Trash2 aria-hidden />
      </Button>
      <ConfirmDialog
        open={open}
        onOpenChange={setOpen}
        title={t("automations.deleteTitle", { title: automation.title })}
        description={t("automations.deleteDescription")}
        confirmLabel={t("common.delete")}
        pending={remove.isPending}
        onConfirm={confirm}
      />
    </>
  );
}
