"use client";

import { useTranslations } from "next-intl";
import { useRef } from "react";
import { Controller, type UseFormReturn } from "react-hook-form";

import type { CatalogueEvent } from "@/entities/automation";
import type { Workflow } from "@/entities/workflow";
import { FormField } from "@/shared/ui/form-field";
import { TemplateInput } from "@/shared/ui/template-input";

import { insertAt, tokenOf, unknownPlaceholders, unknownRanges } from "../model/placeholders";
import type { RunFields } from "../model/run-fields";
import { SELECT } from "./select";

export type WorkflowActionProps = { form: UseFormReturn<RunFields>; event: CatalogueEvent; workflows: Workflow[] };

export function WorkflowAction({ form, event, workflows }: WorkflowActionProps) {
  const t = useTranslations();
  const focus = useRef<{ name: string; cursor: number | null } | null>(null);
  const chosen = form.watch("workflow");
  const inputs = form.watch("inputs");
  const workflow = workflows.find((candidate) => candidate.id === chosen);
  const errors = form.formState.errors;
  const allowed = event.fields.map((field) => field.name);
  const insert = (field: string) => {
    const target = focus.current ?? (workflow?.inputs[0] ? { name: workflow.inputs[0].name, cursor: null } : null);
    if (target === null) {
      return;
    }
    const next = insertAt(inputs[target.name] ?? "", target.cursor, tokenOf(field));
    form.setValue(`inputs.${target.name}`, next.text, { shouldDirty: true, shouldValidate: true });
    focus.current = { name: target.name, cursor: next.cursor };
  };
  const suggestions = event.fields.map((field) => ({ value: field.name, example: field.sample }));
  return (
    <div className="grid gap-4">
      <FormField id="automation-workflow" label={t("automationBuilder.workflow")} hint={t("automationBuilder.workflowHint")} error={errors.workflow?.message}>
        <select id="automation-workflow" className={SELECT} {...form.register("workflow")}>
          <option value="">{t("automationBuilder.chooseWorkflow")}</option>
          {chosen !== "" && !workflow ? <option value={chosen}>{chosen}</option> : null}
          {workflows.map((candidate) => (
            <option key={candidate.id} value={candidate.id}>
              {candidate.title}
            </option>
          ))}
        </select>
      </FormField>
      {workflow && workflow.inputs.length === 0 ? <p className="text-sm text-muted-foreground">{t("automationBuilder.noWorkflowInputs")}</p> : null}
      {workflow?.inputs.map(({ name, type }) => {
        const message = errors.inputs?.[name]?.message;
        const [unknown] = unknownPlaceholders(inputs[name] ?? "", allowed);
        return (
          <FormField
            key={name}
            id={`automation-input-${name}`}
            label={t("automationBuilder.workflowInput", { name })}
            hint={type === "text" ? undefined : t("automationBuilder.typedInput", { type: t(`workflowInputs.types.${type}`) })}
            error={message === "validation.automationPlaceholder" ? t("validation.automationPlaceholder", { field: unknown ?? "" }) : message}
          >
            <Controller
              control={form.control}
              name={`inputs.${name}`}
              render={({ field }) => (
                <TemplateInput
                  id={`automation-input-${name}`}
                  value={field.value ?? ""}
                  onChange={field.onChange}
                  onBlur={field.onBlur}
                  suggestions={suggestions}
                  problems={unknownRanges(field.value ?? "", allowed).map((range) => ({ ...range, message: t("validation.automationPlaceholder", { field: range.name }) }))}
                  onCaret={(cursor) => {
                    focus.current = { name, cursor };
                  }}
                />
              )}
            />
          </FormField>
        );
      })}
      {workflow && workflow.inputs.length > 0 ? (
        <div className="grid gap-2">
          <p className="text-xs font-medium text-muted-foreground">{t("automationBuilder.fields")}</p>
          <div className="flex flex-wrap gap-1.5">
            {event.fields.map((field) => (
              <button
                key={field.name}
                type="button"
                onMouseDown={(press) => press.preventDefault()}
                onClick={() => insert(field.name)}
                className="rounded-full border border-glass-edge bg-glass-tint px-2.5 py-1 font-mono text-xs hover:bg-accent"
              >
                {field.name}
              </button>
            ))}
          </div>
        </div>
      ) : null}
    </div>
  );
}
