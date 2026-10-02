"use client";

import { Trash2 } from "lucide-react";
import { useTranslations } from "next-intl";
import { useState } from "react";
import { toast } from "sonner";

import { useCan } from "@/entities/session";
import { type Service, useDeleteService } from "@/entities/service";
import { ConflictError } from "@/shared/api";
import { ConfirmDialog } from "@/shared/ui/confirm-dialog";
import { Button } from "@/shared/ui/kit";

export type DeleteServiceButtonProps = { service: Service; revision: string | null; onDeleted?: () => void; labelled?: boolean };

function DeleteServiceButtonAllowed({ service, revision, onDeleted, labelled = false }: DeleteServiceButtonProps) {
  const t = useTranslations();
  const remove = useDeleteService();
  const [open, setOpen] = useState(false);
  const confirm = async () => {
    try {
      await remove.mutateAsync({ id: service.id, revision });
      toast.success(t("services.deleted"));
      setOpen(false);
      onDeleted?.();
    } catch (error) {
      toast.error(t(error instanceof ConflictError ? "errors.conflict" : "errors.generic"));
      setOpen(false);
    }
  };
  return (
    <>
      {labelled ? (
        <Button variant="ghost" size="sm" className="text-destructive hover:text-destructive" onClick={() => setOpen(true)}>
          <Trash2 className="size-4" aria-hidden />
          {t("common.delete")}
        </Button>
      ) : (
        <Button variant="ghost" size="icon" aria-label={t("common.delete")} onClick={() => setOpen(true)}>
          <Trash2 aria-hidden />
        </Button>
      )}
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

export function DeleteServiceButton(props: DeleteServiceButtonProps) {
  const can = useCan();
  return can("services", "delete") ? <DeleteServiceButtonAllowed {...props} /> : null;
}
