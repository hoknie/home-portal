"use client";

import { useTranslations } from "next-intl";
import { Controller, type Control, type FieldValues, type Path } from "react-hook-form";

import { FormField } from "@/shared/ui/form-field";
import { TagInput } from "@/shared/ui/tag-input";

export type TagsFieldProps<Form extends FieldValues> = {
  id: string;
  control: Control<Form>;
  name: Path<Form>;
  suggestions: string[];
  error?: string;
};

export function TagsField<Form extends FieldValues>({ id, control, name, suggestions, error }: TagsFieldProps<Form>) {
  const t = useTranslations();
  return (
    <FormField id={id} label={t("tags.label")} hint={t("tags.hint")} optional error={error}>
      <Controller
        control={control}
        name={name}
        render={({ field }) => (
          <TagInput
            id={id}
            values={field.value as string[]}
            onChange={field.onChange}
            onBlur={field.onBlur}
            suggestions={suggestions}
            removeLabel={(value) => t("tagInput.remove", { value })}
            createLabel={(value) => t("tagInput.create", { value })}
          />
        )}
      />
    </FormField>
  );
}
