"use client";

import { useTranslations } from "next-intl";

import { type Step, chosenOf, withChoice } from "@/entities/workflow";
import { cn } from "@/shared/lib/cn";

export type ExclusiveChoiceProps = { step: Step; group: string[]; onChange: (step: Step) => void };

export function ExclusiveChoice({ step, group, onChange }: ExclusiveChoiceProps) {
  const t = useTranslations("workflowEditor.exclusive");
  const current = chosenOf(step, group);
  return (
    <div role="radiogroup" aria-label={t(`label.${step.kind}` as "label.loop")} className="inline-flex w-fit flex-wrap rounded-lg border border-glass-edge p-0.5">
      {group.map((field) => (
        <button
          key={field}
          type="button"
          role="radio"
          aria-checked={field === current}
          className={cn("rounded-md px-2.5 py-1 text-xs", field === current ? "bg-primary text-primary-foreground" : "text-muted-foreground hover:bg-accent")}
          onClick={() => (field === current ? undefined : onChange(withChoice(step, group, field)))}
        >
          {t(`fields.${field}` as "fields.repeat")}
        </button>
      ))}
    </div>
  );
}
