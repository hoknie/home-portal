"use client";

import { Trash2 } from "lucide-react";
import { useTranslations } from "next-intl";
import { useState } from "react";
import { toast } from "sonner";

import { type Service, useDeleteService } from "@/entities/service";
import { ConflictError } from "@/shared/api";
import { ConfirmDialog } from "@/shared/ui/confirm-dialog";
import { Button } from "@/shared/ui/primitives";

export type DeleteServiceButtonProps = { service: Service; revision: string | null };

export function DeleteServiceButton({ service, revision }: DeleteServiceButtonProps) {
  const t = useTranslations();
  const remove = useDeleteService();
  const [open, setOpen] = useState(false);
  const confirm = async () => {
    try {
      await remove.mutateAsync({ id: service.id, revision });
      toast.success(t("services.deleted"));
      setOpen(false);
    } catch (error) {
      toast.error(t(error instanceof ConflictError ? "errors.conflict" : "errors.generic"));
      setOpen(false);
    }
  };
  return (
    <>
      <Button variant="ghost" size="icon" aria-label={t("common.delete")} onClick={() => setOpen(true)}>
        <Trash2 aria-hidden />
      </Button>
      <ConfirmDialog
        open={open}
        onOpenChange={setOpen}
        title={t("services.deleteTitle", { name: service.name })}
        description={t("services.deleteDescription")}
        confirmLabel={t("common.delete")}
        pending={remove.isPending}
        onConfirm={confirm}
      />
    </>
  );
}
