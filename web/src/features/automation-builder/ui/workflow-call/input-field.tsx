"use client";

import { useTranslations } from "next-intl";

import type { CatalogueEvent } from "@/entities/automation";
import { type InputDeclaration, InputValueField } from "@/entities/workflow";
import { FormField, Label, Switch, TemplateInput } from "@/shared/ui/kit";

import { unknownPlaceholders, unknownRanges } from "../../model/placeholders";
import { type InputEntry, switched, templated } from "../../model/workflow-call";

export type InputFieldProps = {
  id: string;
  declaration: InputDeclaration;
  entry: InputEntry;
  event: CatalogueEvent;
  error: string | undefined;
  onChange: (entry: InputEntry) => void;
  onCaret: (cursor: number | null) => void;
};

export function InputField({ id, declaration, entry, event, error, onChange, onCaret }: InputFieldProps) {
  const t = useTranslations();
  const { name, type, description } = declaration;
  const allowed = event.fields.map((field) => field.name);
  const template = templated(entry, declaration);
  const label = t("automationBuilder.workflowInput", { name });
  const [unknown] = unknownPlaceholders(entry.text, allowed);
  const hints = [
    description,
    type === "text" ? null : t("automationBuilder.inputType", { type: t(`workflowInputs.types.${type}`) }),
    declaration.default === null ? null : t("automationBuilder.inputDefault", { value: JSON.stringify(declaration.default) }),
  ].filter(Boolean);
  return (
    <FormField
      id={id}
      label={label}
      hint={hints.join(" · ") || undefined}
      error={error === "validation.automationPlaceholder" ? t("validation.automationPlaceholder", { field: unknown ?? "" }) : error}
    >
      <div className="grid gap-2">
        {type === "text" ? null : (
          <div className="flex items-center gap-2">
            <Switch id={`${id}-template`} checked={template} onCheckedChange={() => onChange(switched(entry, declaration))} />
            <Label htmlFor={`${id}-template`} className="text-xs font-normal text-muted-foreground">
              {t("automationBuilder.templateMode", { name })}
            </Label>
          </div>
        )}
        {template ? (
          <TemplateInput
            id={id}
            aria-label={label}
            value={entry.text}
            onChange={(text) => onChange({ ...entry, template: true, text })}
            suggestions={event.fields.map((field) => ({ value: field.name, example: field.sample }))}
            problems={unknownRanges(entry.text, allowed).map((range) => ({ ...range, message: t("validation.automationPlaceholder", { field: range.name }) }))}
            placeholder={declaration.default === null ? undefined : JSON.stringify(declaration.default)}
            onCaret={onCaret}
          />
        ) : (
          <InputValueField id={id} label={label} type={type} value={entry.value} onChange={(value) => onChange({ ...entry, value })} />
        )}
      </div>
    </FormField>
  );
}
