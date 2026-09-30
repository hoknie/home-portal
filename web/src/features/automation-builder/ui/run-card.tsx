"use client";

import { FolderOpen } from "lucide-react";
import { useTranslations } from "next-intl";
import type { UseFormReturn } from "react-hook-form";

import { type CatalogueEvent, ScriptProblems, type Scripts } from "@/entities/automation";
import { HeaderProblems } from "@/entities/script";
import { FormField } from "@/shared/ui/form-field";
import { Button, Input } from "@/shared/ui/primitives";
import { SectionCard } from "@/shared/ui/section-card";

import type { Workflow } from "@/entities/workflow";

import { ACTIONS, type RunFields } from "../model/run-fields";
import type { WorkflowCallFields } from "../model/workflow-call";
import { ArgumentList } from "./argument-list";
import { SELECT } from "./select";
import { WorkflowCall } from "./workflow-call/workflow-call";

export type RunCardProps = { form: UseFormReturn<RunFields>; event: CatalogueEvent; scripts: Scripts; workflows?: Workflow[]; offerWorkflow?: boolean; idPrefix?: string; showChoice?: boolean };

export function RunCard({ form, event, scripts, workflows, offerWorkflow = false, idPrefix = "automation", showChoice = true }: RunCardProps) {
  const t = useTranslations("automationBuilder");
  const errors = form.formState.errors;
  const chosen = form.watch("script");
  const action = workflows ? form.watch("action") : "script";
  const picked = scripts.scripts.find((script) => script.path === chosen);
  const known = picked !== undefined;
  const choice =
    showChoice && workflows && (offerWorkflow || action === "workflow") ? (
      <div role="radiogroup" aria-label={t("action")} className="flex w-fit gap-1 rounded-lg border border-glass-edge p-1">
        {ACTIONS.map((name) => (
          <Button
            key={name}
            type="button"
            size="sm"
            role="radio"
            aria-checked={action === name}
            variant={action === name ? "default" : "ghost"}
            onClick={() => form.setValue("action", name, { shouldDirty: true, shouldValidate: true })}
          >
            {t(`actions.${name}`)}
          </Button>
        ))}
      </div>
    ) : null;
  if (workflows && action === "workflow") {
    return (
      <SectionCard title={t("run")} description={t("runWorkflowDescription")}>
        <div className="grid gap-5">
          {choice}
          <WorkflowCall form={form as unknown as UseFormReturn<WorkflowCallFields>} event={event} workflows={workflows} idPrefix={idPrefix} />
        </div>
      </SectionCard>
    );
  }
  return (
    <SectionCard title={t("run")} description={t("runDescription")}>
      <div className="grid gap-5">
        {choice}
        {scripts.scripts.length === 0 ? (
          <p role="note" className="flex items-start gap-2 text-sm text-muted-foreground">
            <FolderOpen className="mt-0.5 size-4 shrink-0" aria-hidden />
            {t(scripts.exists ? "noScripts" : "scriptsMissing", { directory: scripts.directory })}
          </p>
        ) : null}
        <div className="grid gap-4 sm:grid-cols-[1fr_10rem]">
          <FormField id="automation-script" label={t("script")} hint={t("scriptHint", { directory: scripts.directory })} error={errors.script?.message}>
            <select id="automation-script" className={SELECT} {...form.register("script")}>
              <option value="">{t("chooseScript")}</option>
              {chosen !== "" && !known ? <option value={chosen}>{chosen}</option> : null}
              {scripts.scripts.map((script) => (
                <option key={script.path} value={script.path} disabled={!script.runnable}>
                  {script.runnable ? script.path : t("scriptUnrunnable", { path: script.path, problem: script.problem ?? "" })}
                </option>
              ))}
            </select>
          </FormField>
          <FormField id="automation-timeout" label={t("timeout")} hint={t("timeoutHint")} error={errors.timeout_seconds?.message}>
            <Input id="automation-timeout" type="number" min={1} max={3600} inputMode="numeric" {...form.register("timeout_seconds", { valueAsNumber: true })} />
          </FormField>
        </div>
        {picked?.description ? <p className="-mt-2 text-sm text-muted-foreground">{picked.description}</p> : null}
        <ScriptProblems scripts={scripts} />
        <HeaderProblems problems={picked?.argument_problems ?? []} />
        <ArgumentList key={chosen} form={form} event={event} declared={picked?.arguments ?? []} />
      </div>
    </SectionCard>
  );
}
