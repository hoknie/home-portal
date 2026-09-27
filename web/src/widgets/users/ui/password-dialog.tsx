"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import { KeyRound } from "lucide-react";
import { useTranslations } from "next-intl";
import { useState } from "react";
import { useForm } from "react-hook-form";
import { toast } from "sonner";

import { type PasswordForm, passwordFormSchema, useChangePassword } from "@/entities/user";
import { RequestError, ValidationError } from "@/shared/api";
import { FormField } from "@/shared/ui/form-field";
import { Button, Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, Input } from "@/shared/ui/primitives";

export type PasswordDialogProps = { name: string; revision: string | null; disabled: boolean };

const EMPTY: PasswordForm = { password: "", repeat: "" };

export function PasswordDialog({ name, revision, disabled }: PasswordDialogProps) {
  const t = useTranslations();
  const [open, setOpen] = useState(false);
  const change = useChangePassword();
  const form = useForm<PasswordForm>({ resolver: zodResolver(passwordFormSchema), defaultValues: EMPTY });
  const errors = form.formState.errors;
  const submit = form.handleSubmit(async (values) => {
    try {
      await change.mutateAsync({ name, password: values.password, revision });
      toast.success(t("users.passwordChanged", { name }));
      form.reset(EMPTY);
      setOpen(false);
    } catch (error) {
      if (error instanceof ValidationError) {
        for (const { field, message } of error.fields) {
          if (field === "password") {
            form.setError("password", { message });
          }
        }
        return;
      }
      toast.error(t("users.refused", { message: error instanceof RequestError ? error.message : String(error) }));
    }
  });
  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <Button
        type="button"
        variant="ghost"
        size="sm"
        disabled={disabled}
        title={disabled ? t("users.moduleOff") : undefined}
        onClick={() => setOpen(true)}
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
