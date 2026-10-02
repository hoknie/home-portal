"use client";

import { useTranslations } from "next-intl";

import { useAutomations } from "@/entities/automation";
import { useWorkflows } from "@/entities/workflow";

import { TemplateField } from "./template-field";
import { NativeSelect } from "@/shared/ui/kit";

type Action = { id?: string; automation?: string; fields?: Record<string, string>; workflow?: string; inputs?: Record<string, unknown>; refresh?: boolean; link?: string };

type Kind = "automation" | "workflow" | "refresh" | "link";

function kindOf(action: Action): Kind {
  return action.automation !== undefined ? "automation" : action.workflow !== undefined ? "workflow" : action.link !== undefined ? "link" : "refresh";
}

export function ActionEditor({ id, action, errors, onChange }: { id: string; action: Action; errors: Record<string, string>; onChange: (action: Action) => void }) {
  const t = useTranslations("layoutEditor.blocks.action");
  const automations = useAutomations().data?.data.automations ?? [];
  const workflows = useWorkflows().data?.data.workflows ?? [];
  const kind = kindOf(action);
  const keep = action.id === undefined ? {} : { id: action.id };
  const choose = (next: Kind) => {
    if (next === "automation") {
      onChange({ ...keep, automation: automations[0]?.id ?? "" });
    } else if (next === "workflow") {
      onChange({ ...keep, workflow: workflows[0]?.id ?? "" });
    } else if (next === "link") {
      onChange({ ...keep, link: "https://" });
    } else {
      onChange({ ...keep, refresh: true });
    }
  };
  const chosenWorkflow = workflows.find((workflow) => workflow.id === action.workflow);
  return (
    <fieldset className="grid gap-2 rounded-lg border border-dashed p-3">
      <legend className="px-1 text-xs font-medium">{t("label")}</legend>
      <NativeSelect aria-label={t("label")} value={kind} onChange={(event) => choose(event.target.value as Kind)}>
        {(["automation", "workflow", "refresh", "link"] as const).map((value) => (
          <option key={value} value={value}>
            {t(`kinds.${value}`)}
          </option>
        ))}
      </NativeSelect>
      {errors[""] ? <p role="alert" className="text-xs text-destructive">{errors[""]}</p> : null}
      {kind === "automation" ? (
        <>
          <NativeSelect aria-label={t("automation")} value={action.automation} onChange={(event) => onChange({ ...keep, automation: event.target.value })}>
            {automations.map((automation) => (
              <option key={automation.id} value={automation.id}>
                {automation.title}
              </option>
            ))}
          </NativeSelect>
          {errors.automation ? <p role="alert" className="text-xs text-destructive">{errors.automation}</p> : null}
        </>
      ) : null}
      {kind === "workflow" ? (
        <>
          <NativeSelect aria-label={t("workflow")} value={action.workflow} onChange={(event) => onChange({ ...keep, workflow: event.target.value, inputs: {} })}>
            {workflows.map((workflow) => (
              <option key={workflow.id} value={workflow.id}>
                {workflow.title}
              </option>
            ))}
          </NativeSelect>
          {(chosenWorkflow?.inputs ?? []).map((input) => (
            <TemplateField
              key={input.name}
              id={`${id}-input-${input.name}`}
              label={`${t("input")}: ${input.name}`}
              value={typeof action.inputs?.[input.name] === "string" ? (action.inputs[input.name] as string) : ""}
              error={errors[`inputs.${input.name}`]}
              optional
              onChange={(value) => {
                const inputs = { ...(action.inputs ?? {}) };
                if (value === "") {
                  delete inputs[input.name];
                } else {
                  inputs[input.name] = value;
                }
                onChange({ ...action, inputs });
              }}
            />
          ))}
        </>
      ) : null}
      {kind === "link" ? <TemplateField id={`${id}-link`} label={t("link")} value={action.link ?? ""} error={errors.link} onChange={(link) => onChange({ ...keep, link })} /> : null}
    </fieldset>
  );
}
