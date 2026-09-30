"use client";

import { useTranslations } from "next-intl";
import { useState } from "react";
import { toast } from "sonner";

import { type Group, type MatrixRow, type Rights, useChangeGroup, useCreateGroup } from "@/entities/user";
import { RequestError, ValidationError } from "@/shared/api";
import { FormField } from "@/shared/ui/form-field";
import { Button, Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, Input } from "@/shared/ui/primitives";

import { MatrixEditor } from "./matrix-editor";

export type GroupDialogProps = { group: Group | null; matrix: MatrixRow[]; revision: string | null; open: boolean; onOpenChange: (open: boolean) => void };

export function GroupDialog({ group, matrix, revision, open, onOpenChange }: GroupDialogProps) {
  const t = useTranslations("users.groups");
  const common = useTranslations("common");
  const [name, setName] = useState(group?.name ?? "");
  const [rights, setRights] = useState<Rights>(group?.rights ?? {});
  const [problems, setProblems] = useState<Record<string, string>>({});
  const create = useCreateGroup();
  const change = useChangeGroup();
  const pending = create.isPending || change.isPending;
  const save = async () => {
    setProblems({});
    try {
      if (group === null) {
        await create.mutateAsync({ name, rights, revision });
        toast.success(t("created", { name }));
      } else {
        await change.mutateAsync({ current: group.name, name, rights, revision });
        toast.success(t("saved", { name }));
      }
      onOpenChange(false);
    } catch (error) {
      if (error instanceof ValidationError) {
        setProblems(Object.fromEntries(error.fields.map((field) => [field.field, field.message])));
        return;
      }
      setProblems({ form: error instanceof RequestError ? error.message : String(error) });
    }
  };
  const rightsProblem = Object.entries(problems)
    .filter(([field]) => field.startsWith("rights"))
    .map(([field, message]) => `${field}: ${message}`)
    .join("; ");
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent closeLabel={common("close")} className="sm:max-w-3xl">
        <DialogHeader>
          <DialogTitle>{group === null ? t("addTitle") : t("editTitle", { name: group.name })}</DialogTitle>
          <DialogDescription>{t("dialogDescription")}</DialogDescription>
        </DialogHeader>
        <div className="grid gap-4">
          <FormField id="group-name" label={t("name")} error={problems.name}>
            <Input id="group-name" autoComplete="off" spellCheck={false} value={name} onChange={(event) => setName(event.target.value)} />
          </FormField>
          <MatrixEditor matrix={matrix} rights={rights} onChange={setRights} />
          {rightsProblem || problems.form ? (
            <p role="alert" className="text-sm text-destructive">
              {rightsProblem || problems.form}
            </p>
          ) : null}
        </div>
        <DialogFooter>
          <Button type="button" disabled={pending} onClick={() => void save()}>
            {common("save")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
