"use client";

import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import { BareButton, Label, Switch } from "@/shared/ui/kit";

import type { ArgumentValues } from "../model/arguments";
import type { HeaderProblem, ScriptArgument } from "../model/schema";

export type ArgumentTextProps = {
  id: string;
  label: string;
  value: string;
  placeholder?: string;
  onChange: (value: string) => void;
};

export type DeclaredArgumentsProps = {
  idPrefix: string;
  declared: ScriptArgument[];
  values: ArgumentValues;
  onChange: (values: ArgumentValues) => void;
  renderText: (argument: ScriptArgument, props: ArgumentTextProps) => ReactNode;
};

export function DeclaredArguments({ idPrefix, declared, values, onChange, renderText }: DeclaredArgumentsProps) {
  const t = useTranslations("scriptArguments");
  const set = (name: string, value: string | boolean) => onChange({ ...values, [name]: value });
  return (
    <ul aria-label={t("declared")} className="grid gap-3">
      {declared.map((argument) => {
        const id = `${idPrefix}-${argument.name.replace(/^-+/, "")}`;
        const current = values[argument.name];
        return (
          <li key={argument.name} className="grid gap-1">
            {argument.type === "flag" ? (
              <div className="flex items-center gap-2">
                <Switch id={id} checked={current === true} onCheckedChange={(checked) => set(argument.name, checked)} />
                <Label htmlFor={id} className="font-mono">
                  {argument.name}
                </Label>
              </div>
            ) : (
              <>
                <Label htmlFor={id} className="flex items-baseline gap-2">
                  <span className="font-mono">{argument.name}</span>
                  {argument.required ? <span className="text-xs font-normal text-muted-foreground">{t("required")}</span> : null}
                </Label>
                {renderText(argument, {
                  id,
                  label: argument.name,
                  value: typeof current === "string" ? current : "",
                  placeholder: argument.default ?? undefined,
                  onChange: (value) => set(argument.name, value),
                })}
                {argument.choices.length > 0 ? (
                  <div role="group" aria-label={t("choices")} className="flex flex-wrap gap-1.5">
                    {argument.choices.map((choice) => (
                      <BareButton
                        key={choice}
                        aria-pressed={current === choice}
                        onClick={() => set(argument.name, choice)}
                        className="rounded-full border border-glass-edge bg-glass-tint px-2.5 py-0.5 font-mono text-xs hover:bg-accent aria-pressed:border-primary aria-pressed:text-primary"
                      >
                        {choice}
                      </BareButton>
                    ))}
                  </div>
                ) : null}
              </>
            )}
            {argument.description || argument.default ? (
              <p className="text-xs text-muted-foreground">
                {[argument.description, argument.default ? t("defaultIs", { value: argument.default }) : ""].filter(Boolean).join(" · ")}
              </p>
            ) : null}
          </li>
        );
      })}
    </ul>
  );
}

export function HeaderProblems({ problems }: { problems: HeaderProblem[] }) {
  const t = useTranslations("scriptArguments");
  if (problems.length === 0) {
    return null;
  }
  return (
    <div role="note" className="grid gap-1 rounded-lg border border-status-degraded/40 p-2 text-xs">
      <p className="font-medium">{t("headerProblems")}</p>
      <ul className="grid gap-0.5 font-mono text-muted-foreground">
        {problems.map((problem) => (
          <li key={`${problem.line}-${problem.message}`}>{problem.message}</li>
        ))}
      </ul>
    </div>
  );
}

export type ArgumentModeProps = {
  asList: boolean;
  fits: boolean;
  onChange: (asList: boolean) => void;
};

export function ArgumentMode({ asList, fits, onChange }: ArgumentModeProps) {
  const t = useTranslations("scriptArguments");
  return (
    <div className="flex flex-wrap items-center gap-2 text-xs">
      {!fits ? (
        <p role="note" className="text-muted-foreground">
          {t("doesNotFit")}
        </p>
      ) : null}
      {fits ? (
        <BareButton className="text-primary underline-offset-2 hover:underline" onClick={() => onChange(!asList)}>
          {asList ? t("editAsFields") : t("editAsList")}
        </BareButton>
      ) : null}
    </div>
  );
}
