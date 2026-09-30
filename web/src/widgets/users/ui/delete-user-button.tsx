"use client";

import { Trash2 } from "lucide-react";
import { useTranslations } from "next-intl";
import { useState } from "react";
import { toast } from "sonner";

import { type User, type Users, deletable, lastAdmin, useDeleteUser } from "@/entities/user";
import { RequestError } from "@/shared/api";
import { ConfirmDialog } from "@/shared/ui/confirm-dialog";
import { Button } from "@/shared/ui/primitives";

export type DeleteUserButtonProps = { user: User; users: Users; revision: string | null };

export function DeleteUserButton({ user, users, revision }: DeleteUserButtonProps) {
  const t = useTranslations("users");
  const [open, setOpen] = useState(false);
  const remove = useDeleteUser();
  const reason = !users.editable
    ? t("moduleOff")
    : user.you
      ? t("cannotDeleteYourself")
      : users.users.length <= 1
        ? t("cannotDeleteLast")
        : lastAdmin(user, users)
          ? t("cannotDeleteLastAdmin")
          : undefined;
  const confirm = async () => {
    try {
      await remove.mutateAsync({ name: user.name, revision });
      toast.success(t("deleted", { name: user.name }));
    } catch (error) {
      toast.error(t("refused", { message: error instanceof RequestError ? error.message : String(error) }));
    }
    setOpen(false);
  };
  return (
    <>
      <Button
        type="button"
        variant="ghost"
        size="sm"
        disabled={!deletable(user, users)}
        title={reason}
        onClick={() => setOpen(true)}
      >
        <Trash2 aria-hidden />
        {t("delete")}
      </Button>
      <ConfirmDialog
        open={open}
        onOpenChange={setOpen}
        title={t("deleteTitle", { name: user.name })}
        description={t("deleteDescription", { name: user.name })}
        confirmLabel={t("delete")}
        pending={remove.isPending}
        onConfirm={() => void confirm()}
      />
    </>
  );
}
