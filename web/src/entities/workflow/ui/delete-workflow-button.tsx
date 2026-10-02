"use client";

import { Trash2 } from "lucide-react";
import { useTranslations } from "next-intl";
import { useState } from "react";
import { toast } from "sonner";

import { useDeleteWorkflow } from "../model/queries";
import type { Workflow } from "../model/schema";
import { ConflictError } from "@/shared/api";
import { ConfirmDialog } from "@/shared/ui/confirm-dialog";
import { Button } from "@/shared/ui/kit";

export type DeleteWorkflowButtonProps = { workflow: Workflow; revision: string | null; labelled?: boolean; onDeleted?: () => void };

export function DeleteWorkflowButton({ workflow, revision, labelled = false, onDeleted }: DeleteWorkflowButtonProps) {
  const t = useTranslations();
  const remove = useDeleteWorkflow();
  const [open, setOpen] = useState(false);
  const users = workflow.used_by.map((usage) => usage.title).join(", ");
  const confirm = async () => {
    try {
      await remove.mutateAsync({ id: workflow.id, revision });
      toast.success(t("workflows.deleted"));
      setOpen(false);
      onDeleted?.();
    } catch (error) {
      toast.error(t(error instanceof ConflictError ? "errors.conflict" : "errors.generic"));
      setOpen(false);
    }
  };
  return (
    <>
      <span title={users === "" ? undefined : t("workflows.deleteBlocked", { users })}>
        <Button type="button" variant="ghost" size={labelled ? "sm" : "icon"} aria-label={labelled ? undefined : t("common.delete")} disabled={users !== ""} onClick={() => setOpen(true)}>
          <Trash2 aria-hidden />
          {labelled ? t("common.delete") : null}
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
