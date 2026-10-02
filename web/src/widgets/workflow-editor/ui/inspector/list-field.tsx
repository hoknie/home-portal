"use client";

import { ArrowUp, Plus, Trash2 } from "lucide-react";
import { useTranslations } from "next-intl";

import type { Path } from "@/entities/workflow";
import { Button, Label } from "@/shared/ui/kit";

import { StepTemplateInput } from "./template-field";

export type ListFieldProps = { path: Path; field: string; label: string; hint?: string; values: string[]; onChange: (values: string[]) => void };

export function ListField({ path, field, label, hint, values, onChange }: ListFieldProps) {
  const t = useTranslations("workflowEditor");
  return (
    <div className="grid gap-2">
      <Label>{label}</Label>
      {hint ? <p className="text-xs text-muted-foreground">{hint}</p> : null}
      {values.map((value, index) => (
        <div key={`${field}-${index}`} className="flex min-w-0 items-start gap-1">
          <div className="min-w-0 flex-1">
            <StepTemplateInput
              path={path}
              field={`${field}[${index}]`}
              label={t("item", { label, number: index + 1 })}
              value={value}
              onChange={(changed) => onChange(values.map((current, position) => (position === index ? changed : current)))}
            />
          </div>
          <Button
            type="button"
            variant="ghost"
            size="icon"
            aria-label={t("moveItemUp", { number: index + 1 })}
            disabled={index === 0}
            onClick={() => onChange(values.map((current, position) => (position === index - 1 ? values[index] : position === index ? values[index - 1] : current)))}
          >
            <ArrowUp aria-hidden />
          </Button>
          <Button type="button" variant="ghost" size="icon" aria-label={t("removeItem")} onClick={() => onChange(values.filter((_, position) => position !== index))}>
            <Trash2 aria-hidden />
          </Button>
        </div>
      ))}
      <Button type="button" variant="outline" size="sm" className="w-fit" onClick={() => onChange([...values, ""])}>
        <Plus aria-hidden />
        {t("addItem")}
      </Button>
    </div>
  );
}
