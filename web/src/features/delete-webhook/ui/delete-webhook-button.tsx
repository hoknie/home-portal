"use client";

import { Trash2 } from "lucide-react";
import { useTranslations } from "next-intl";
import { useState } from "react";
import { toast } from "sonner";

import { type Webhook, useDeleteWebhook } from "@/entities/webhook";
import { ConflictError } from "@/shared/api";
import { ConfirmDialog } from "@/shared/ui/confirm-dialog";
import { Button } from "@/shared/ui/primitives";

export type DeleteWebhookButtonProps = { webhook: Webhook; revision: string | null };

export function DeleteWebhookButton({ webhook, revision }: DeleteWebhookButtonProps) {
  const t = useTranslations();
  const remove = useDeleteWebhook();
  const [open, setOpen] = useState(false);
  const confirm = async () => {
    try {
      await remove.mutateAsync({ id: webhook.id, revision });
      toast.success(t("webhooks.deleted"));
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
        title={t("webhooks.deleteTitle", { title: webhook.title })}
        description={t("webhooks.deleteDescription")}
        confirmLabel={t("common.delete")}
        pending={remove.isPending}
        onConfirm={() => void confirm()}
      />
    </>
  );
}
