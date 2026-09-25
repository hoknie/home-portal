"use client";

import { useTranslations } from "next-intl";

import { Label } from "@/shared/ui/primitives";

export type ChoiceListProps = {
  id: string;
  legend: string;
  hint?: string;
  choices: { value: string; label: string }[];
  values: string[];
  onChange: (values: string[]) => void;
  error?: string;
};

export function ChoiceList({ id, legend, hint, choices, values, onChange, error }: ChoiceListProps) {
  const t = useTranslations();
  const message = error && t.has(error as Parameters<typeof t.has>[0]) ? t(error as Parameters<typeof t>[0]) : error;
  return (
    <div role="group" aria-labelledby={`${id}-label`} className="grid content-start gap-2">
      <Label asChild>
        <span id={`${id}-label`}>{legend}</span>
      </Label>
      <div className="flex flex-wrap gap-x-4">
        {choices.map((choice) => (
          <label key={choice.value} className="flex min-h-9 items-center gap-2 text-sm">
            <input
              type="checkbox"
              className="size-4 accent-primary"
              checked={values.includes(choice.value)}
              onChange={(event) => onChange(event.target.checked ? [...values, choice.value] : values.filter((value) => value !== choice.value))}
            />
            {choice.label}
          </label>
        ))}
      </div>
      {hint ? <p className="text-xs text-muted-foreground">{hint}</p> : null}
      {message ? (
        <p role="alert" className="text-sm text-destructive">
          {message}
        </p>
      ) : null}
    </div>
  );
}
