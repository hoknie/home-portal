"use client";

import { Plus, Trash2 } from "lucide-react";
import { useTranslations } from "next-intl";

import type { Path } from "@/entities/workflow";
import { Button, Label } from "@/shared/ui/primitives";
import { TemplateInput, type TemplateSuggestion } from "@/shared/ui/template-input";

import { useEditor } from "../../model/editor-context";
import { StepTemplateInput, useGroupLabel, useTemplateProblems, useTranslatedSuggestions } from "./template-field";

export type TableFieldProps = {
  path: Path;
  field: string;
  label: string;
  hint?: string;
  values: Record<string, string>;
  fixedKeys?: string[];
  keySuggestions?: TemplateSuggestion[];
  onChange: (values: Record<string, string>) => void;
  keyProblem?: (key: string) => string | undefined;
  templateKeys?: boolean;
};

export function TableField({ path, field, label, hint, values, fixedKeys, keySuggestions = [], onChange, keyProblem, templateKeys = false }: TableFieldProps) {
  const t = useTranslations("workflowEditor");
  const editor = useEditor();
  const translate = useTranslatedSuggestions();
  const groupLabel = useGroupLabel();
  const problemsOf = useTemplateProblems();
  const entries = fixedKeys ? fixedKeys.map((key) => [key, values[key] ?? ""] as const) : Object.entries(values);
  const rename = (from: string, to: string) => onChange(Object.fromEntries(Object.entries(values).map(([key, value]) => (key === from ? [to, value] : [key, value]))));
  return (
    <div className="grid gap-2">
      <Label>{label}</Label>
      {hint ? <p className="text-xs text-muted-foreground">{hint}</p> : null}
      {entries.length === 0 && fixedKeys ? <p className="text-xs text-muted-foreground">{t("noInputs")}</p> : null}
      {entries.map(([key, value], index) => (
        <div key={`${field}-${index}`} className="grid min-w-0 gap-1 sm:grid-cols-[9rem_1fr_auto] sm:items-start">
          {fixedKeys ? (
            <span className="self-center font-mono text-xs">{key}</span>
          ) : (
            <div className="grid gap-1">
              <TemplateInput
                aria-label={t("key", { label, number: index + 1 })}
                aria-invalid={key !== "" && keyProblem?.(key) ? true : undefined}
                trigger="always"
                value={key}
                suggestions={keySuggestions}
                templateSuggestions={templateKeys ? translate(editor.suggestionsFor(path, `${field}.${key}`)) : undefined}
                problems={templateKeys ? problemsOf(path, `${field}.${key}`, key) : []}
                groupLabel={groupLabel}
                onChange={(next) => rename(key, next)}
              />
              {key !== "" && keyProblem?.(key) ? (
                <p role="alert" className="text-xs text-destructive">
                  {keyProblem(key)}
                </p>
              ) : null}
            </div>
          )}
          <StepTemplateInput path={path} field={`${field}.${key}`} label={t("valueOf", { key: key || String(index + 1) })} value={value} onChange={(changed) => onChange({ ...values, [key]: changed })} />
          {fixedKeys ? null : (
            <Button type="button" variant="ghost" size="icon" aria-label={t("removeItem")} onClick={() => onChange(Object.fromEntries(Object.entries(values).filter(([name]) => name !== key)))}>
              <Trash2 aria-hidden />
            </Button>
          )}
        </div>
      ))}
      {fixedKeys ? null : (
        <Button type="button" variant="outline" size="sm" className="w-fit" disabled={"" in values} onClick={() => onChange({ ...values, "": "" })}>
          <Plus aria-hidden />
          {t("addItem")}
        </Button>
      )}
    </div>
  );
}
