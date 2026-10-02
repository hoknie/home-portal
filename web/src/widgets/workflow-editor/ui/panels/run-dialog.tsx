"use client";

import { Play } from "lucide-react";
import { useTranslations } from "next-intl";
import { useState } from "react";

import { type InputDeclaration, InputsForm, initialValues, runValues } from "@/entities/workflow";
import { Button, Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/shared/ui/kit";

export type RunDialogProps = {
  open: boolean;
  title: string;
  inputs: InputDeclaration[];
  pending: boolean;
  onRun: (inputs: Record<string, unknown>) => void;
  onClose: () => void;
};

export function RunDialog({ open, title, inputs, pending, onRun, onClose }: RunDialogProps) {
  const t = useTranslations();
  const [values, setValues] = useState<Record<string, unknown>>(() => initialValues(inputs));
  return (
    <Dialog open={open} onOpenChange={(next) => (next ? undefined : onClose())}>
      <DialogContent closeLabel={t("common.close")}>
        <DialogHeader>
          <DialogTitle>{t("workflows.runNowTitle", { title })}</DialogTitle>
          <DialogDescription>{t("workflows.runNowDescription")}</DialogDescription>
        </DialogHeader>
        <InputsForm idPrefix="editor-run" inputs={inputs} values={values} onChange={setValues} />
        <DialogFooter>
          <Button variant="outline" onClick={onClose} disabled={pending}>
            {t("common.cancel")}
          </Button>
          <Button onClick={() => onRun(runValues(inputs, values))} disabled={pending}>
            <Play aria-hidden />
            {t("workflows.runNow")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
