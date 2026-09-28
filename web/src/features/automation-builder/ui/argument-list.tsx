"use client";

import { ArrowDown, ArrowUp, Plus, X } from "lucide-react";
import { useTranslations } from "next-intl";
import { useRef } from "react";
import { Controller, type UseFormReturn, useFieldArray } from "react-hook-form";

import { type CatalogueEvent, messageKeyOf } from "@/entities/automation";
import { Button, Label } from "@/shared/ui/primitives";
import { TemplateInput } from "@/shared/ui/template-input";

import type { RunFields } from "../model/run-fields";
import { insertAt, tokenOf, unknownPlaceholders, unknownRanges } from "../model/placeholders";

export type ArgumentListProps = { form: UseFormReturn<RunFields>; event: CatalogueEvent };

export function ArgumentList({ form, event }: ArgumentListProps) {
  const t = useTranslations();
  const list = useFieldArray({ control: form.control, name: "args" });
  const focus = useRef<{ index: number; cursor: number | null } | null>(null);
  const errors = form.formState.errors.args;
  const label = (name: string) => {
    const key = `automationFields.${messageKeyOf(name)}` as Parameters<typeof t>[0];
    return t.has(key as Parameters<typeof t.has>[0]) ? t(key) : name;
  };
  const allowed = event.fields.map((field) => field.name);
  const suggestions = event.fields.map((field) => ({ value: field.name, description: label(field.name), example: field.sample }));
  const problemsOf = (text: string) =>
    unknownRanges(text, allowed).map((range) => ({ ...range, message: t("validation.automationPlaceholder", { field: range.name }) }));
  const explain = (index: number, message: string) => {
    if (message === "validation.automationPlaceholder") {
      const [field] = unknownPlaceholders(form.getValues(`args.${index}.value`), allowed);
      return t("validation.automationPlaceholder", { field: field ?? "" });
    }
    return t.has(message as Parameters<typeof t.has>[0]) ? t(message as Parameters<typeof t>[0]) : message;
  };
  const insert = (field: string) => {
    const token = tokenOf(field);
    const target = focus.current;
    if (target === null || target.index >= list.fields.length) {
      list.append({ value: token });
      focus.current = { index: list.fields.length, cursor: token.length };
      return;
    }
    const current = form.getValues(`args.${target.index}.value`);
    const next = insertAt(current, target.cursor, token);
    form.setValue(`args.${target.index}.value`, next.text, { shouldDirty: true, shouldValidate: true });
    focus.current = { index: target.index, cursor: next.cursor };
  };
  return (
    <div className="grid gap-3">
      <div className="grid gap-1">
        <Label asChild>
          <span id="automation-arguments-label">{t("automationBuilder.arguments")}</span>
        </Label>
        <p className="text-xs text-muted-foreground">{t("automationBuilder.argumentsHint")}</p>
      </div>
      <ol aria-labelledby="automation-arguments-label" className="grid gap-2">
        {list.fields.map((item, index) => {
          const message = errors?.[index]?.value?.message;
          return (
            <li key={item.id} className="grid gap-1">
              <div className="flex items-center gap-1">
                <span className="w-6 shrink-0 text-right font-mono text-xs text-muted-foreground">{index + 1}</span>
                <Controller
                  control={form.control}
                  name={`args.${index}.value`}
                  render={({ field }) => (
                    <TemplateInput
                      aria-label={t("automationBuilder.argumentLabel", { number: index + 1 })}
                      aria-invalid={message ? true : undefined}
                      value={field.value}
                      onChange={field.onChange}
                      onBlur={field.onBlur}
                      suggestions={suggestions}
                      problems={problemsOf(field.value)}
                      onCaret={(cursor) => {
                        focus.current = { index, cursor };
                      }}
                    />
                  )}
                />
                <Button type="button" variant="ghost" size="icon" aria-label={t("automationBuilder.moveUp")} disabled={index === 0} onClick={() => list.move(index, index - 1)}>
                  <ArrowUp aria-hidden />
                </Button>
                <Button
                  type="button"
                  variant="ghost"
                  size="icon"
                  aria-label={t("automationBuilder.moveDown")}
                  disabled={index === list.fields.length - 1}
                  onClick={() => list.move(index, index + 1)}
                >
                  <ArrowDown aria-hidden />
                </Button>
                <Button type="button" variant="ghost" size="icon" aria-label={t("automationBuilder.removeArgument")} onClick={() => list.remove(index)}>
                  <X aria-hidden />
                </Button>
              </div>
              {message ? (
                <p role="alert" className="pl-7 text-sm text-destructive">
                  {explain(index, message)}
                </p>
              ) : null}
            </li>
          );
        })}
      </ol>
      <div>
        <Button type="button" variant="outline" size="sm" onClick={() => list.append({ value: "" })}>
          <Plus aria-hidden />
          {t("automationBuilder.addArgument")}
        </Button>
      </div>
      <div className="grid gap-2">
        <p className="text-xs font-medium text-muted-foreground">{t("automationBuilder.fields")}</p>
        <div className="flex flex-wrap gap-1.5">
          {event.fields.map((field) => (
            <button
              key={field.name}
              type="button"
              title={label(field.name)}
              onMouseDown={(press) => press.preventDefault()}
              onClick={() => insert(field.name)}
              className="rounded-full border border-glass-edge bg-glass-tint px-2.5 py-1 font-mono text-xs hover:bg-accent"
            >
              {field.name}
            </button>
          ))}
        </div>
      </div>
    </div>
  );
}
