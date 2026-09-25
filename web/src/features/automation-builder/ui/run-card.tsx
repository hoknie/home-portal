"use client";

import { FolderOpen } from "lucide-react";
import { useTranslations } from "next-intl";
import type { UseFormReturn } from "react-hook-form";

import { type CatalogueEvent, ScriptProblems, type Scripts } from "@/entities/automation";
import { FormField } from "@/shared/ui/form-field";
import { Input } from "@/shared/ui/primitives";
import { SectionCard } from "@/shared/ui/section-card";

import type { RunFields } from "../model/run-fields";
import { ArgumentList } from "./argument-list";
import { SELECT } from "./select";

export type RunCardProps = { form: UseFormReturn<RunFields>; event: CatalogueEvent; scripts: Scripts };

export function RunCard({ form, event, scripts }: RunCardProps) {
  const t = useTranslations("automationBuilder");
  const errors = form.formState.errors;
  const chosen = form.watch("script");
  const known = scripts.scripts.some((script) => script.path === chosen);
  return (
    <SectionCard title={t("run")} description={t("runDescription")}>
      <div className="grid gap-5">
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
        <ScriptProblems scripts={scripts} />
        <ArgumentList form={form} event={event} />
      </div>
    </SectionCard>
  );
}
