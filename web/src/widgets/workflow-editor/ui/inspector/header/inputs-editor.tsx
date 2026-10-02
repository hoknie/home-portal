"use client";

import { Plus, Trash2 } from "lucide-react";
import { useTranslations } from "next-intl";

import { INPUT_TYPES, type InputDeclaration, type InputType, InputValueField, emptyValue, plainInput } from "@/entities/workflow";
import { Button, Input, Label, NativeSelect } from "@/shared/ui/kit";


export type InputsEditorProps = { inputs: InputDeclaration[]; problemAt: (at: string) => string | undefined; onChange: (key: string, inputs: InputDeclaration[]) => void };

function meaningful(type: InputType, value: unknown): unknown {
  if (value === "" || value === null || value === undefined) {
    return null;
  }
  if ((type === "list" && Array.isArray(value) && value.length === 0) || (type === "object" && typeof value === "object" && Object.keys(value as object).length === 0)) {
    return null;
  }
  return value;
}

export function InputsEditor({ inputs, problemAt, onChange }: InputsEditorProps) {
  const t = useTranslations();
  const change = (index: number, key: string, next: Partial<InputDeclaration>) =>
    onChange(
      `inputs[${index}].${key}`,
      inputs.map((current, position) => (position === index ? { ...current, ...next } : current)),
    );
  return (
    <div className="grid gap-2">
      <Label>{t("workflowEditor.inputs")}</Label>
      <p className="text-xs text-muted-foreground">{t("workflowEditor.inputsHint")}</p>
      {inputs.map((input, index) => {
        const problem = problemAt(`inputs[${index}]`) ?? problemAt(`inputs[${index}].default`);
        return (
          <fieldset key={`input-${index}`} className="grid gap-1.5 rounded-lg border border-glass-edge p-2" aria-label={t("workflowEditor.inputName", { number: index + 1 })}>
            <div className="flex gap-1">
              <Input
                aria-label={t("workflowEditor.inputName", { number: index + 1 })}
                aria-invalid={problem ? true : undefined}
                className="font-mono"
                value={input.name}
                onChange={(event) => change(index, "name", { name: event.target.value })}
              />
              <NativeSelect
                aria-label={t("workflowInputs.type", { number: index + 1 })}
                className="w-28"
                value={input.type}
                onChange={(event) => change(index, "type", { type: event.target.value as InputType, default: null })}
              >
                {INPUT_TYPES.map((type) => (
                  <option key={type} value={type}>
                    {t(`workflowInputs.types.${type}`)}
                  </option>
                ))}
              </NativeSelect>
              <Button
                type="button"
                variant="ghost"
                size="icon"
                aria-label={t("workflowEditor.removeInput", { name: input.name })}
                onClick={() => onChange("inputs", inputs.filter((_, position) => position !== index))}
              >
                <Trash2 aria-hidden />
              </Button>
            </div>
            <InputValueField
              id={`workflow-input-default-${index}`}
              label={t("workflowInputs.default", { number: index + 1 })}
              type={input.type}
              value={input.default ?? emptyValue(input.type)}
              onChange={(value) => change(index, "default", { default: meaningful(input.type, value) })}
            />
            <Input
              aria-label={t("workflowInputs.description", { number: index + 1 })}
              placeholder={t("workflowInputs.descriptionPlaceholder")}
              value={input.description ?? ""}
              onChange={(event) => change(index, "description", { description: event.target.value === "" ? null : event.target.value })}
            />
            {problem ? (
              <p role="alert" className="text-sm text-destructive">
                {problem}
              </p>
            ) : null}
          </fieldset>
        );
      })}
      <Button type="button" variant="outline" size="sm" className="w-fit" onClick={() => onChange("inputs", [...inputs, plainInput("")])}>
        <Plus aria-hidden />
        {t("workflowEditor.addInput")}
      </Button>
    </div>
  );
}
