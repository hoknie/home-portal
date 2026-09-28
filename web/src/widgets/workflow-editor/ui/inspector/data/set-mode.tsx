"use client";

import { useTranslations } from "next-intl";

import type { Step } from "@/entities/workflow";
import { cn } from "@/shared/lib/cn";

export const SET_FORMS = ["value", "json", "list", "object"] as const;

export type SetForm = (typeof SET_FORMS)[number];

export function setFormOf(step: Step): SetForm {
  return SET_FORMS.find((form) => (step as Record<string, unknown>)[form] !== undefined) ?? "value";
}

const EMPTY: Record<SetForm, unknown> = { value: "", json: "", list: [""], object: { "": "" } };

export function SetMode({ step, onChange }: { step: Step; onChange: (step: Step) => void }) {
  const t = useTranslations("workflowEditor.setForms");
  const current = setFormOf(step);
  const choose = (form: SetForm) => {
    const copy: Record<string, unknown> = { ...step };
    SET_FORMS.forEach((name) => delete copy[name]);
    copy[form] = EMPTY[form];
    onChange(copy as Step);
  };
  return (
    <div role="radiogroup" aria-label={t("label")} className="inline-flex w-fit rounded-lg border border-glass-edge p-0.5">
      {SET_FORMS.map((form) => (
        <button
          key={form}
          type="button"
          role="radio"
          aria-checked={form === current}
          className={cn("rounded-md px-2.5 py-1 text-xs", form === current ? "bg-primary text-primary-foreground" : "text-muted-foreground hover:bg-accent")}
          onClick={() => (form === current ? undefined : choose(form))}
        >
          {t(form)}
        </button>
      ))}
    </div>
  );
}
