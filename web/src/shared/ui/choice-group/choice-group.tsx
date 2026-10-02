"use client";

import type { KeyboardEvent, ReactNode } from "react";

import { cn } from "@/shared/lib/cn";

export type Choice<Value extends string> = { value: Value; label: string; swatch?: ReactNode };

export type ChoiceGroupProps<Value extends string> = {
  label: string;
  value: Value;
  choices: Choice<Value>[];
  onChange: (value: Value) => void;
};

export function ChoiceGroup<Value extends string>({ label, value, choices, onChange }: ChoiceGroupProps<Value>) {
  const step = (event: KeyboardEvent<HTMLDivElement>) => {
    const delta = event.key === "ArrowRight" || event.key === "ArrowDown" ? 1 : event.key === "ArrowLeft" || event.key === "ArrowUp" ? -1 : 0;
    if (delta === 0) {
      return;
    }
    event.preventDefault();
    const index = choices.findIndex((choice) => choice.value === value);
    const next = choices[(index + delta + choices.length) % choices.length];
    onChange(next.value);
    const buttons = event.currentTarget.querySelectorAll<HTMLButtonElement>("[role=radio]");
    buttons[choices.indexOf(next)]?.focus();
  };
  return (
    <fieldset className="grid gap-2">
      <legend className="mb-2 text-sm font-medium">{label}</legend>
      <div role="radiogroup" aria-label={label} className="flex flex-wrap gap-2" onKeyDown={step}>
        {choices.map((choice) => {
          const chosen = choice.value === value;
          return (
            <button
              key={choice.value}
              type="button"
              role="radio"
              aria-checked={chosen}
              tabIndex={chosen ? 0 : -1}
              onClick={() => onChange(choice.value)}
              className={cn(
                "flex min-w-16 flex-col items-center gap-1.5 rounded-lg border p-2 text-xs transition-colors outline-none focus-visible:ring-[3px] focus-visible:ring-ring/50",
                chosen ? "border-primary bg-primary/10 text-foreground" : "border-border text-muted-foreground hover:bg-muted",
              )}
            >
              {choice.swatch}
              {choice.label}
            </button>
          );
        })}
      </div>
    </fieldset>
  );
}
