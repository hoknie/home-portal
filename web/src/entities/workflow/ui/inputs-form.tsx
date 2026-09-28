"use client";

import { useTranslations } from "next-intl";

import { FormField } from "@/shared/ui/form-field";

import { namedInputs } from "../model/inputs";
import type { InputDeclaration } from "../model/schema";
import { InputValueField } from "./input-value-field";

export type InputsFormProps = { idPrefix: string; inputs: InputDeclaration[]; values: Record<string, unknown>; onChange: (values: Record<string, unknown>) => void };

export function InputsForm({ idPrefix, inputs, values, onChange }: InputsFormProps) {
  const t = useTranslations("workflowInputs");
  return (
    <div className="grid gap-3">
      {namedInputs(inputs).map((input) => {
        const id = `${idPrefix}-${input.name}`;
        const hint = [input.type === "text" ? null : t(`types.${input.type}`), input.description].filter(Boolean).join(" · ") || undefined;
        return (
          <FormField key={input.name} id={id} label={input.name} hint={hint}>
            <InputValueField id={id} label={input.name} type={input.type} value={values[input.name]} onChange={(value) => onChange({ ...values, [input.name]: value })} />
          </FormField>
        );
      })}
    </div>
  );
}
