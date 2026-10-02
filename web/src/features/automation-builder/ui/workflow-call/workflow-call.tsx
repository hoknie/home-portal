"use client";

import { useTranslations } from "next-intl";
import { useRef } from "react";
import type { UseFormReturn } from "react-hook-form";

import type { CatalogueEvent } from "@/entities/automation";
import type { Workflow } from "@/entities/workflow";
import { BareButton, FormField, NativeSelect } from "@/shared/ui/kit";

import { insertAt, tokenOf } from "../../model/placeholders";
import { EMPTY_ENTRY, type WorkflowCallFields, emptyEntry, kept, textOf } from "../../model/workflow-call";
import { InputField } from "./input-field";

export type WorkflowCallProps = { form: UseFormReturn<WorkflowCallFields>; event: CatalogueEvent; workflows: Workflow[]; idPrefix: string };

export function WorkflowCall({ form, event, workflows, idPrefix }: WorkflowCallProps) {
  const t = useTranslations();
  const focus = useRef<{ name: string; cursor: number | null } | null>(null);
  const chosen = form.watch("workflow");
  const inputs = form.watch("inputs");
  const workflow = workflows.find((candidate) => candidate.id === chosen);
  const errors = form.formState.errors;
  const set = (name: string, entry: WorkflowCallFields["inputs"][string]) => form.setValue(`inputs.${name}`, entry, { shouldDirty: true, shouldValidate: true });
  const insert = (field: string) => {
    const target = focus.current ?? (workflow?.inputs[0] ? { name: workflow.inputs[0].name, cursor: null } : null);
    if (target === null) {
      return;
    }
    const current = inputs[target.name] ?? EMPTY_ENTRY;
    const next = insertAt(textOf(current), target.cursor, tokenOf(field));
    set(target.name, { ...current, template: true, text: next.text });
    focus.current = { name: target.name, cursor: next.cursor };
  };
  const pick = (id: string) => {
    const declarations = workflows.find((candidate) => candidate.id === id)?.inputs ?? [];
    form.setValue("workflow", id, { shouldDirty: true, shouldValidate: true });
    form.setValue("inputs", kept(form.getValues("inputs"), declarations), { shouldDirty: true, shouldValidate: true });
    focus.current = null;
  };
  return (
    <div className="grid gap-4">
      <FormField id={`${idPrefix}-workflow`} label={t("automationBuilder.workflow")} hint={t("automationBuilder.workflowHint")} error={errors.workflow?.message}>
        <NativeSelect id={`${idPrefix}-workflow`} value={chosen} onChange={(change) => pick(change.target.value)}>
          <option value="">{t("automationBuilder.chooseWorkflow")}</option>
          {chosen !== "" && !workflow ? <option value={chosen}>{chosen}</option> : null}
          {workflows.map((candidate) => (
            <option key={candidate.id} value={candidate.id}>
              {candidate.title}
            </option>
          ))}
        </NativeSelect>
      </FormField>
      {workflow?.description ? <p className="-mt-2 text-sm text-muted-foreground">{workflow.description}</p> : null}
      {workflow && workflow.inputs.length === 0 ? <p className="text-sm text-muted-foreground">{t("automationBuilder.noWorkflowInputs")}</p> : null}
      {workflow?.inputs.map((declaration) => (
        <InputField
          key={declaration.name}
          id={`${idPrefix}-input-${declaration.name}`}
          declaration={declaration}
          entry={inputs[declaration.name] ?? emptyEntry(declaration)}
          event={event}
          error={errors.inputs?.[declaration.name]?.message}
          onChange={(entry) => set(declaration.name, entry)}
          onCaret={(cursor) => {
            focus.current = { name: declaration.name, cursor };
          }}
        />
      ))}
      {workflow && workflow.inputs.length > 0 ? (
        <div className="grid gap-2">
          <p className="text-xs font-medium text-muted-foreground">{t("automationBuilder.fields")}</p>
          <div className="flex flex-wrap gap-1.5">
            {event.fields.map((field) => (
              <BareButton
                key={field.name}
                onMouseDown={(press) => press.preventDefault()}
                onClick={() => insert(field.name)}
                className="rounded-full border border-glass-edge bg-glass-tint px-2.5 py-1 font-mono text-xs hover:bg-accent"
              >
                {field.name}
              </BareButton>
            ))}
          </div>
        </div>
      ) : null}
    </div>
  );
}
