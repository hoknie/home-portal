"use client";

import { Trash2 } from "lucide-react";
import { useTranslations } from "next-intl";
import { useState } from "react";
import { toast } from "sonner";

import { type Workflow, useDeleteWorkflow } from "@/entities/workflow";
import { ConflictError } from "@/shared/api";
import { ConfirmDialog } from "@/shared/ui/confirm-dialog";
import { Button } from "@/shared/ui/primitives";

export function DeleteWorkflowButton({ workflow, revision }: { workflow: Workflow; revision: string | null }) {
  const t = useTranslations();
  const remove = useDeleteWorkflow();
  const [open, setOpen] = useState(false);
  const users = workflow.used_by.map((usage) => usage.title).join(", ");
  const confirm = async () => {
    try {
      await remove.mutateAsync({ id: workflow.id, revision });
      toast.success(t("workflows.deleted"));
    } catch (error) {
      toast.error(t(error instanceof ConflictError ? "errors.conflict" : "errors.generic"));
    }
    setOpen(false);
  };
  return (
    <>
      <span title={users === "" ? undefined : t("workflows.deleteBlocked", { users })}>
        <Button variant="ghost" size="icon" aria-label={t("common.delete")} disabled={users !== ""} onClick={() => setOpen(true)}>
          <Trash2 aria-hidden />
        </Button>
      </span>
      <ConfirmDialog
        open={open}
        onOpenChange={setOpen}
        title={t("workflows.deleteTitle", { title: workflow.title })}
        description={t("workflows.deleteDescription")}
        confirmLabel={t("common.delete")}
        pending={remove.isPending}
        onConfirm={() => void confirm()}
      />
    </>
  );
}
