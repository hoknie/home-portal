"use client";

import { Play } from "lucide-react";
import { useTranslations } from "next-intl";

import { useScripts } from "@/entities/automation";
import { useWorkflows } from "@/entities/workflow";
import { CopyLine } from "@/shared/ui/copy-line";
import { Button, FormField, Heading, Input, NativeSelect } from "@/shared/ui/kit";
import { JsonView } from "@/shared/ui/json-view";

import { pathsOf } from "../../model/data-paths";
import { ScriptArguments, WorkflowInputs } from "./source-arguments";
import type { WidgetPreviewState } from "../../model/use-widget-preview";
import { ChoiceGroup } from "@/shared/ui/choice-group";

type Source = { workflow?: string; inputs?: Record<string, unknown>; script?: string; args?: string[]; timeout_seconds?: number };

export type DataTabProps = {
  settings: Record<string, unknown>;
  errors: Record<string, string>;
  preview: WidgetPreviewState;
  onChange: (settings: Record<string, unknown>) => void;
};

const DEFAULT_REFRESH = 300;

function sourceKind(source: Source | undefined): "none" | "workflow" | "script" {
  return source?.workflow !== undefined ? "workflow" : source?.script !== undefined ? "script" : "none";
}

export function DataTab({ settings, errors, preview, onChange }: DataTabProps) {
  const t = useTranslations("layoutEditor.data");
  const workflows = useWorkflows().data?.data.workflows ?? [];
  const scripts = useScripts().data?.scripts ?? [];
  const source = (settings.source ?? undefined) as Source | undefined;
  const kind = sourceKind(source);
  const setSource = (next: Source | undefined) => {
    const rest = { ...settings };
    delete rest.source;
    onChange(next === undefined ? rest : { ...rest, source: next });
  };
  const chosenWorkflow = workflows.find((workflow) => workflow.id === source?.workflow);
  const data = preview.preview?.data;
  return (
    <div className="@container grid gap-5">
      <ChoiceGroup
        label={t("source")}
        value={kind}
        onChange={(chosen) => setSource(chosen === "none" ? undefined : chosen === "workflow" ? { workflow: workflows[0]?.id ?? "" } : { script: scripts[0]?.path ?? "" })}
        choices={(["none", "workflow", "script"] as const).map((value) => ({ value, label: t(`kinds.${value}`) }))}
      />
      {kind === "workflow" ? (
        <div className="grid gap-4">
          <FormField id="source-workflow" label={t("workflow")} error={errors["source.workflow"] ?? errors.source}>
            <NativeSelect id="source-workflow" value={source?.workflow ?? ""} onChange={(event) => setSource({ workflow: event.target.value, inputs: {} })}>
              {workflows.map((workflow) => (
                <option key={workflow.id} value={workflow.id}>
                  {workflow.title}
                </option>
              ))}
            </NativeSelect>
          </FormField>
          <WorkflowInputs workflow={chosenWorkflow} values={source?.inputs ?? {}} errors={errors} onChange={(inputs) => setSource({ ...source, inputs })} />
        </div>
      ) : null}
      {kind === "script" ? (
        <div className="grid gap-4 @sm:grid-cols-2">
          <FormField id="source-script" label={t("script")} error={errors["source.script"] ?? errors.source}>
            <NativeSelect id="source-script" value={source?.script ?? ""} onChange={(event) => setSource({ ...source, script: event.target.value })}>
              {scripts.map((script) => (
                <option key={script.path} value={script.path}>
                  {script.path}
                </option>
              ))}
            </NativeSelect>
          </FormField>
          <FormField id="source-timeout" label={t("timeout")} error={errors["source.timeout_seconds"]} optional>
            <Input
              id="source-timeout"
              type="number"
              min={1}
              max={300}
              value={source?.timeout_seconds ?? ""}
              onChange={(event) => setSource({ ...source, timeout_seconds: event.target.value === "" ? undefined : Number(event.target.value) })}
            />
          </FormField>
          <div className="@sm:col-span-2">
            <ScriptArguments key={source?.script} script={scripts.find((script) => script.path === source?.script)} args={source?.args ?? []} onChange={(args) => setSource({ ...source, args })} />
          </div>
        </div>
      ) : null}
      <FormField id="refresh-seconds" label={t("refresh")} hint={t("refreshHint")} error={errors.refresh_seconds} optional>
        <Input
          id="refresh-seconds"
          type="number"
          min={30}
          max={86400}
          placeholder={String(DEFAULT_REFRESH)}
          value={typeof settings.refresh_seconds === "number" ? settings.refresh_seconds : ""}
          onChange={(event) => {
            const rest = { ...settings };
            delete rest.refresh_seconds;
            onChange(event.target.value === "" ? rest : { ...rest, refresh_seconds: Number(event.target.value) });
          }}
        />
      </FormField>
      {kind !== "none" ? (
        <div className="grid gap-3">
          <div className="flex flex-wrap items-center gap-3">
            <Button type="button" variant="outline" size="sm" onClick={preview.run} disabled={preview.running}>
              <Play aria-hidden />
              {preview.running ? t("running") : t("runNow")}
            </Button>
            {preview.problem ? <p role="alert" className="text-sm text-destructive">{preview.problem}</p> : null}
            {preview.forbidden ? <p role="alert" className="text-sm text-destructive">{t("runForbidden")}</p> : null}
          </div>
          {data !== undefined && data !== null ? (
            <section className="grid gap-2" aria-label={t("tree")}>
              <Heading level="group" as="h3" className="text-sm font-medium">{t("tree")}</Heading>
              <JsonView value={data} openLevels={2} className="max-h-72 overflow-auto rounded-xl border border-glass-edge bg-muted/40 p-3 font-mono text-xs" />
              <div className="grid max-h-40 gap-1 overflow-auto" data-paths="">
                {pathsOf(data, "data").map((path) => (
                  <CopyLine key={path} text={`{{${path}}}`} />
                ))}
              </div>
            </section>
          ) : (
            <p className="text-xs text-muted-foreground">{t("noData")}</p>
          )}
        </div>
      ) : null}
    </div>
  );
}
