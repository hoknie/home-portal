"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import { KeyRound } from "lucide-react";
import { useTranslations } from "next-intl";
import { useState } from "react";
import { useForm } from "react-hook-form";
import { toast } from "sonner";

import { type PasswordForm, passwordFormSchema, useChangePassword } from "@/entities/user";
import { ConflictError, RequestError, ValidationError } from "@/shared/api";
import { useEditorRevision } from "@/shared/lib/editor-revision";
import { ConflictNotice } from "@/shared/ui/conflict-notice";
import { FormField } from "@/shared/ui/form-field";
import { Button, Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, Input } from "@/shared/ui/primitives";

export type PasswordDialogProps = { name: string; revision: string | null; disabled: boolean; own: boolean };

const EMPTY: PasswordForm = { password: "", repeat: "", current: "" };
const CURRENT_FIELD = "current_password";

export function PasswordDialog({ name, revision, disabled, own }: PasswordDialogProps) {
  const t = useTranslations();
  const [open, setOpen] = useState(false);
  const change = useChangePassword();
  const form = useForm<PasswordForm>({ resolver: zodResolver(passwordFormSchema), defaultValues: EMPTY });
  const errors = form.formState.errors;
  const [conflict, setConflict] = useState(false);
  const held = useEditorRevision(revision);
  const reload = () => {
    held.catchUp();
    form.reset(EMPTY);
    setConflict(false);
  };
  const send = (at: string | null) => form.handleSubmit(async (values) => {
    setConflict(false);
    if (own && values.current === "") {
      form.setError("current", { message: "validation.userCurrentPassword" });
      return;
    }
    try {
      await change.mutateAsync({ name, password: values.password, current: own ? values.current : null, revision: at });
      toast.success(t("users.passwordChanged", { name }));
      form.reset(EMPTY);
      setOpen(false);
    } catch (error) {
      if (error instanceof ValidationError) {
        for (const { field, message } of error.fields) {
          if (field === "password") {
            form.setError("password", { message });
          }
          if (field === CURRENT_FIELD) {
            form.setError("current", { message: "validation.userCurrentPassword" });
          }
        }
        return;
      }
      if (error instanceof ConflictError) {
        setConflict(true);
        return;
      }
      toast.error(t("users.refused", { message: error instanceof RequestError ? error.message : String(error) }));
    }
  });
  const submit = send(held.revision);
  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <Button
        type="button"
        variant="ghost"
        size="sm"
        disabled={disabled}
        title={disabled ? t("users.moduleOff") : undefined}
        onClick={() => {
          reload();
          setOpen(true);
        }}
      >
        <KeyRound aria-hidden />
        {t("users.changePassword")}
      </Button>
      <DialogContent closeLabel={t("common.close")}>
        <DialogHeader>
          <DialogTitle>{t("users.passwordTitle", { name })}</DialogTitle>
          <DialogDescription>{t("users.passwordDescription", { name })}</DialogDescription>
        </DialogHeader>
        <form onSubmit={submit} className="grid gap-4" noValidate>
          {conflict ? <ConflictNotice pending={held.latest === held.revision || change.isPending} onReload={reload} onOverwrite={() => void send(held.catchUp())()} /> : null}
          {own ? (
            <FormField id={`current-${name}`} label={t("users.currentPassword")} error={errors.current?.message}>
              <Input id={`current-${name}`} type="password" autoComplete="current-password" {...form.register("current")} />
            </FormField>
          ) : null}
          <FormField id={`password-${name}`} label={t("users.password")} hint={t("users.passwordHint")} error={errors.password?.message}>
            <Input id={`password-${name}`} type="password" autoComplete="new-password" {...form.register("password")} />
          </FormField>
          <FormField id={`repeat-${name}`} label={t("users.repeat")} error={errors.repeat?.message}>
            <Input id={`repeat-${name}`} type="password" autoComplete="new-password" {...form.register("repeat")} />
          </FormField>
          <DialogFooter>
            <Button type="submit" disabled={change.isPending}>
              {t("users.savePassword")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
