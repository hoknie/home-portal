"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import { UserPlus } from "lucide-react";
import { useTranslations } from "next-intl";
import { useState } from "react";
import { useForm } from "react-hook-form";
import { toast } from "sonner";

import { type NewUserForm, newUserFormSchema, useCreateUser } from "@/entities/user";
import { RequestError, ValidationError } from "@/shared/api";
import { FormField } from "@/shared/ui/form-field";
import { Button, Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, Input } from "@/shared/ui/primitives";

import { GroupSelect, NO_GROUP, groupOf } from "./group-select";

const FORBIDDEN = 403;

export type AddUserDialogProps = { revision: string | null; disabled: boolean };

const EMPTY: NewUserForm = { name: "", password: "", repeat: "" };

export function AddUserDialog({ revision, disabled }: AddUserDialogProps) {
  const t = useTranslations();
  const [open, setOpen] = useState(false);
  const [group, setGroup] = useState(NO_GROUP);
  const [groupProblem, setGroupProblem] = useState<string | null>(null);
  const create = useCreateUser();
  const form = useForm<NewUserForm>({ resolver: zodResolver(newUserFormSchema), defaultValues: EMPTY });
  const errors = form.formState.errors;
  const submit = form.handleSubmit(async (values) => {
    try {
      setGroupProblem(null);
      await create.mutateAsync({ name: values.name, password: values.password, group: groupOf(group), revision });
      toast.success(t("users.created", { name: values.name }));
      form.reset(EMPTY);
      setOpen(false);
    } catch (error) {
      if (error instanceof ValidationError) {
        for (const { field, message } of error.fields) {
          if (field === "name" || field === "password") {
            form.setError(field, { message });
          }
          if (field === "group") {
            setGroupProblem(message);
          }
        }
        return;
      }
      setGroupProblem(error instanceof RequestError && error.status === FORBIDDEN ? t("access.forbidden") : null);
      toast.error(t("users.refused", { message: error instanceof RequestError ? error.message : String(error) }));
    }
  });
  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <Button type="button" disabled={disabled} title={disabled ? t("users.moduleOff") : undefined} onClick={() => setOpen(true)}>
        <UserPlus aria-hidden />
        {t("users.add")}
      </Button>
      <DialogContent closeLabel={t("common.close")}>
        <DialogHeader>
          <DialogTitle>{t("users.addTitle")}</DialogTitle>
          <DialogDescription>{t("users.addDescription")}</DialogDescription>
        </DialogHeader>
        <form onSubmit={submit} className="grid gap-4" noValidate>
          <FormField id="user-name" label={t("users.name")} error={errors.name?.message}>
            <Input id="user-name" autoComplete="off" spellCheck={false} {...form.register("name")} />
          </FormField>
          <FormField id="user-group" label={t("users.group")} error={groupProblem ?? undefined}>
            <GroupSelect id="user-group" value={group} onChange={setGroup} />
          </FormField>
          <FormField id="user-password" label={t("users.password")} hint={t("users.passwordHint")} error={errors.password?.message}>
            <Input id="user-password" type="password" autoComplete="new-password" {...form.register("password")} />
          </FormField>
          <FormField id="user-repeat" label={t("users.repeat")} error={errors.repeat?.message}>
            <Input id="user-repeat" type="password" autoComplete="new-password" {...form.register("repeat")} />
          </FormField>
          <DialogFooter>
            <Button type="submit" disabled={create.isPending}>
              {t("users.create")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
