"use client";

import { Users as UsersIcon } from "lucide-react";
import { useTranslations } from "next-intl";
import { useState } from "react";
import { toast } from "sonner";

import { type User, useChangeUserGroup } from "@/entities/user";
import { ConflictError, RequestError, ValidationError } from "@/shared/api";
import { useEditorRevision } from "@/shared/lib/editor-revision";
import { ConflictNotice } from "@/shared/ui/conflict-notice";
import { FormField } from "@/shared/ui/form-field";
import { Button, Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/shared/ui/primitives";

import { GroupSelect, NO_GROUP, groupOf } from "./group-select";

export type ChangeGroupDialogProps = { user: User; revision: string | null; disabled: boolean };

export function ChangeGroupDialog({ user, revision, disabled }: ChangeGroupDialogProps) {
  const t = useTranslations("users");
  const common = useTranslations("common");
  const [open, setOpen] = useState(false);
  const [value, setValue] = useState(user.group ?? NO_GROUP);
  const [problem, setProblem] = useState<string | null>(null);
  const [conflict, setConflict] = useState(false);
  const change = useChangeUserGroup();
  const held = useEditorRevision(revision);
  const reload = () => {
    held.catchUp();
    setValue(user.group ?? NO_GROUP);
    setConflict(false);
  };
  const save = async (at: string | null = held.revision) => {
    setProblem(null);
    setConflict(false);
    try {
      await change.mutateAsync({ name: user.name, group: groupOf(value), revision: at });
      toast.success(t("groupChanged", { name: user.name }));
      setOpen(false);
    } catch (error) {
      if (error instanceof ValidationError) {
        setProblem(error.fields.map((field) => field.message).join("; "));
        return;
      }
      if (error instanceof ConflictError) {
        setConflict(true);
        return;
      }
      setProblem(error instanceof RequestError ? error.message : String(error));
    }
  };
  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <Button type="button" variant="ghost" size="sm" disabled={disabled} title={disabled ? t("moduleOff") : undefined} onClick={() => {
          reload();
          setOpen(true);
        }}>
        <UsersIcon aria-hidden />
        {t("changeGroup")}
      </Button>
      <DialogContent closeLabel={common("close")}>
        <DialogHeader>
          <DialogTitle>{t("groupTitle", { name: user.name })}</DialogTitle>
          <DialogDescription>{t("groupDescription")}</DialogDescription>
        </DialogHeader>
        {conflict ? <ConflictNotice pending={held.latest === held.revision || change.isPending} onReload={reload} onOverwrite={() => void save(held.catchUp())} /> : null}
        <FormField id={`group-of-${user.name}`} label={t("group")} error={problem ?? undefined}>
          <GroupSelect id={`group-of-${user.name}`} value={value} onChange={setValue} />
        </FormField>
        <DialogFooter>
          <Button type="button" disabled={change.isPending} onClick={() => void save()}>
            {t("saveGroup")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
