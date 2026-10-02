"use client";

import { useTranslations } from "next-intl";
import { useState } from "react";

import { Button, Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle, Input, Label } from "@/shared/ui/primitives";

import { joined } from "../../model/script-usage";

const SELECT = "h-9 w-full rounded-md border border-input bg-glass-tint px-3 text-sm";

export type PlaceDialogProps = {
  open: boolean;
  title: string;
  confirmLabel: string;
  folders: string[];
  folder?: string;
  name?: string;
  withFolder?: boolean;
  pending?: boolean;
  error?: string | null;
  onConfirm: (place: string) => void;
  onOpenChange: (open: boolean) => void;
};

type PlaceFormProps = Omit<PlaceDialogProps, "open" | "title">;

function PlaceForm({
  confirmLabel,
  folders,
  folder = "",
  name = "",
  withFolder = true,
  pending = false,
  error = null,
  onConfirm,
  onOpenChange,
}: PlaceFormProps) {
  const t = useTranslations("scripts");
  const common = useTranslations("common");
  const [chosen, setChosen] = useState(folder);
  const [typed, setTyped] = useState(name);
  return (
    <form
      className="grid gap-4"
      onSubmit={(submit) => {
        submit.preventDefault();
        onConfirm(withFolder ? joined(chosen, typed.trim()) : typed.trim());
      }}
    >
      {withFolder ? (
        <div className="grid gap-1.5">
          <Label htmlFor="script-place-folder">{t("folder")}</Label>
          <select id="script-place-folder" className={SELECT} value={chosen} onChange={(change) => setChosen(change.target.value)}>
            <option value="">{t("topLevel")}</option>
            {folders.map((option) => (
              <option key={option} value={option}>
                {option}
              </option>
            ))}
          </select>
        </div>
      ) : null}
      <div className="grid gap-1.5">
        <Label htmlFor="script-place-name">{t("name")}</Label>
        <Input id="script-place-name" autoFocus value={typed} onChange={(change) => setTyped(change.target.value)} aria-invalid={error ? true : undefined} />
        {error ? (
          <p role="alert" className="text-sm text-destructive">
            {error}
          </p>
        ) : null}
      </div>
      <DialogFooter>
        <Button type="button" variant="outline" onClick={() => onOpenChange(false)} disabled={pending}>
          {common("cancel")}
        </Button>
        <Button type="submit" disabled={pending || typed.trim() === ""}>
          {confirmLabel}
        </Button>
      </DialogFooter>
    </form>
  );
}

export function PlaceDialog({ open, title, ...form }: PlaceDialogProps) {
  const common = useTranslations("common");
  return (
    <Dialog open={open} onOpenChange={form.onOpenChange}>
      <DialogContent closeLabel={common("close")}>
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
        </DialogHeader>
        <PlaceForm {...form} />
      </DialogContent>
    </Dialog>
  );
}
