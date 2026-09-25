"use client";

import { useTranslations } from "next-intl";

import { FormField } from "@/shared/ui/form-field";
import { Input } from "@/shared/ui/primitives";
import { TagInput } from "@/shared/ui/tag-input";

import { SETTINGS, type SettingsField, settingsErrors } from "../model/settings-fields";

export type SettingsFormProps = {
  type: string;
  value: Record<string, unknown>;
  onChange: (value: Record<string, unknown>) => void;
  groups?: string[];
};

const SELECT =
  "h-9 w-full rounded-md border border-input bg-glass-tint px-3 text-sm shadow-xs outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50";

function shown(field: SettingsField, value: unknown) {
  if (field.kind === "list") {
    return Array.isArray(value) ? value.join(", ") : "";
  }
  return value === undefined || value === null ? "" : String(value);
}

function parsed(field: SettingsField, text: string): unknown {
  if (text.trim() === "") {
    return undefined;
  }
  if (field.kind === "number") {
    const number = Number(text.replace(",", "."));
    return Number.isNaN(number) ? text : number;
  }
  if (field.kind === "list") {
    return text
      .split(",")
      .map((item) => item.trim())
      .filter((item) => item !== "");
  }
  return text;
}

export function SettingsForm({ type, value, onChange, groups = [] }: SettingsFormProps) {
  const t = useTranslations();
  const description = SETTINGS[type];
  if (!description) {
    return null;
  }
  const errors = settingsErrors(type, value);
  const put = (field: SettingsField, entry: unknown) => {
    const next = { ...value, [field.key]: entry };
    if (next[field.key] === undefined) {
      delete next[field.key];
    }
    onChange(next);
  };
  const set = (field: SettingsField, text: string) => put(field, parsed(field, text));
  return (
    <div className="grid gap-4">
      {description.fields.map((field) => {
        const id = `setting-${field.key}`;
        return (
          <FormField key={field.key} id={id} label={t(field.label)} optional={field.optional} error={errors[field.key]}>
            {field.kind === "groups" ? (
              <TagInput
                id={id}
                values={Array.isArray(value[field.key]) ? (value[field.key] as string[]) : []}
                onChange={(values) => put(field, values.length === 0 ? undefined : values)}
                suggestions={groups}
                removeLabel={(item) => t("tagInput.remove", { value: item })}
                createLabel={(item) => t("tagInput.create", { value: item })}
              />
            ) : field.kind === "select" ? (
              <select id={id} className={SELECT} value={shown(field, value[field.key])} onChange={(event) => set(field, event.target.value)}>
                <option value="">{t("widgetSettings.default")}</option>
                {(field.options ?? []).map((option) => (
                  <option key={option} value={option}>
                    {option}
                  </option>
                ))}
              </select>
            ) : (
              <Input
                id={id}
                inputMode={field.kind === "number" ? "decimal" : undefined}
                defaultValue={shown(field, value[field.key])}
                onChange={(event) => set(field, event.target.value)}
              />
            )}
          </FormField>
        );
      })}
    </div>
  );
}
