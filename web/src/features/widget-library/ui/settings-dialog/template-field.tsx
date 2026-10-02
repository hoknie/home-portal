"use client";

import { createContext, useContext } from "react";

import { FormField } from "@/shared/ui/form-field";
import { TemplateInput, type TemplateSuggestion } from "@/shared/ui/template-input";

export type Suggesting = {
  names: (items: string | null) => TemplateSuggestion[];
  filters: (subject: string, chain: string) => TemplateSuggestion[];
  caret?: (field: string, caret: number | null) => void;
};

export const SuggestingContext = createContext<Suggesting>({ names: () => [], filters: () => [] });

export type TemplateFieldProps = {
  id: string;
  label: string;
  value: string;
  error?: string;
  optional?: boolean;
  items?: string | null;
  field?: string;
  hint?: string;
  example?: string;
  multiline?: boolean;
  onChange: (value: string) => void;
};

export function TemplateField({ id, label, value, error, optional, items = null, field, hint, example, multiline = false, onChange }: TemplateFieldProps) {
  const suggesting = useContext(SuggestingContext);
  return (
    <FormField id={id} label={label} error={error} optional={optional} hint={hint}>
      <TemplateInput
        id={id}
        value={value}
        onChange={onChange}
        suggestions={suggesting.names(items)}
        filterSuggestions={suggesting.filters}
        multiline={multiline}
        placeholder={example}
        aria-invalid={error ? true : undefined}
        onCaret={field === undefined || suggesting.caret === undefined ? undefined : (caret) => suggesting.caret?.(field, caret)}
      />
    </FormField>
  );
}
