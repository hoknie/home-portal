"use client";

import { useTranslations } from "next-intl";
import { Fragment, type ReactNode } from "react";

import { fromArgs } from "@/entities/script";
import { type KindField, type Step, hiddenFields, operatorName } from "@/entities/workflow";

import { useEditor } from "../../model/editor-context";
import { summaryOf } from "../../model/summary";
import { FALLBACK_ICON, GROUP_TONE, KIND_ICONS } from "../nodes/kind-style";
import { Produces } from "./produces";
import { INVERTED_FIELDS, NESTED_TYPES, useFieldText } from "./step-form";

const TEMPLATE_PART = /(\{\{[^}]*\}\})/g;
const WHOLE_TEMPLATE = /^\{\{[^}]*\}\}$/;
const SKIPPED_TYPES = ["operations", "sample"];

function TemplateText({ text }: { text: string }) {
  return (
    <span className="font-mono text-xs break-all">
      {text.split(TEMPLATE_PART).map((part, index) =>
        WHOLE_TEMPLATE.test(part) ? (
          <span key={index} className="rounded bg-primary/12 px-1 text-foreground">
            {part}
          </span>
        ) : (
          <Fragment key={index}>{part}</Fragment>
        ),
      )}
    </span>
  );
}

function Rows({ rows }: { rows: [string | null, string][] }) {
  return (
    <ul className="grid gap-1">
      {rows.map(([key, value], index) => (
        <li key={`${key ?? index}-${index}`} className="flex min-w-0 gap-2">
          {key === null ? null : <span className="shrink-0 font-mono text-xs text-muted-foreground">{key}</span>}
          <TemplateText text={value} />
        </li>
      ))}
    </ul>
  );
}

function text(value: unknown) {
  return typeof value === "string" ? value : JSON.stringify(value);
}

function Readable({ step, field, value }: { step: Step; field: KindField; value: unknown }): ReactNode {
  const t = useTranslations("workflowEditor.card");
  const editor = useEditor();
  const operators = useTranslations("workflowEditor.condition.operators");
  if (field.type === "boolean") {
    const on = value === true || (value === undefined && field.default === "true");
    return on !== INVERTED_FIELDS.includes(`${step.kind}.${field.name}`) ? t("yes") : t("no");
  }
  if (field.type === "condition") {
    const summary = summaryOf(step, [], [], (operator) => (operators.has(operatorName(operator) as "equals") ? operators(operatorName(operator) as "equals") : operator));
    return <TemplateText text={"text" in summary ? summary.text : text(value)} />;
  }
  if (field.type === "template-list" && Array.isArray(value)) {
    const declared = step.kind === "script" ? (editor.sources.scripts.find((script) => script.path === step.script)?.arguments ?? []) : [];
    const named = declared.length > 0 ? fromArgs(declared, value.map(text)) : null;
    return named ? <Rows rows={Object.entries(named).map(([name, given]) => [name, given === true ? t("yes") : text(given)])} /> : <Rows rows={value.map((item) => [null, text(item)])} />;
  }
  if (field.type === "template-table" && value !== null && typeof value === "object") {
    return <Rows rows={Object.entries(value).map(([key, item]) => [key, text(item)])} />;
  }
  if (field.type === "workflow") {
    return editor.sources.workflows.find((workflow) => workflow.id === value)?.title ?? text(value);
  }
  if (field.type === "automation") {
    return editor.sources.automations.find((automation) => automation.id === value)?.title ?? text(value);
  }
  return <TemplateText text={text(value)} />;
}

function Setting({ step, field }: { step: Step; field: KindField }) {
  const t = useTranslations("workflowEditor.card");
  const label = useFieldText(step.kind, field).label;
  const value = (step as Record<string, unknown>)[field.name];
  const unset = value === undefined || value === null || value === "" || (Array.isArray(value) && value.length === 0);
  if (unset && field.default === null && field.type !== "boolean") {
    return null;
  }
  return (
    <>
      <dt className="text-xs text-muted-foreground">{label}</dt>
      <dd className="min-w-0 text-sm">
        {unset && field.type !== "boolean" ? <TemplateText text={field.default ?? ""} /> : <Readable step={step} field={field} value={value} />}
        {unset ? <span className="ms-1 text-xs text-muted-foreground">{t("byDefault")}</span> : null}
      </dd>
    </>
  );
}

export function StepCard({ step, compact = false }: { step: Step; compact?: boolean }) {
  const t = useTranslations("workflowEditor.card");
  const help = useTranslations("workflowHelp");
  const editor = useEditor();
  const kind = editor.kindOf(step.kind);
  const Icon = KIND_ICONS[step.kind] ?? FALLBACK_ICON;
  const has = (key: string) => help.has(key as "kinds.if.name");
  const hidden = hiddenFields(step, kind);
  const fields = (kind?.fields ?? []).filter((field) => !hidden.has(field.name) && !NESTED_TYPES.includes(field.type) && !SKIPPED_TYPES.includes(field.type));
  return (
    <section className={compact ? "grid gap-2 [&_dd]:text-xs" : "grid gap-4"} aria-label={t("title", { label: step.label ?? step.id })}>
      {compact ? null : (
        <div className="flex items-start gap-3">
          <span className={`grid size-8 shrink-0 place-items-center rounded-lg ${GROUP_TONE[kind?.group ?? "actions"].badge}`}>
            <Icon className="size-4" aria-hidden />
          </span>
          <div className="grid min-w-0 gap-0.5">
            <p className="text-sm font-semibold">{has(`kinds.${step.kind}.name`) ? help(`kinds.${step.kind}.name` as "kinds.if.name") : step.kind}</p>
            {has(`kinds.${step.kind}.description`) ? <p className="text-xs text-muted-foreground">{help(`kinds.${step.kind}.description` as "kinds.if.description")}</p> : null}
          </div>
        </div>
      )}
      <dl className={compact ? "grid grid-cols-[minmax(0,8rem)_minmax(0,1fr)] gap-x-3 gap-y-1" : "grid grid-cols-[minmax(0,9rem)_minmax(0,1fr)] gap-x-3 gap-y-2"}>
        {compact ? null : (
          <>
            <dt className="text-xs text-muted-foreground">{t("label")}</dt>
            <dd className="text-sm">{step.label ?? step.id}</dd>
            <dt className="text-xs text-muted-foreground">{t("id")}</dt>
            <dd className="font-mono text-xs">{step.id}</dd>
          </>
        )}
        {fields.map((field) => (
          <Setting key={field.name} step={step} field={field} />
        ))}
      </dl>
      {compact ? null : <Produces step={step} />}
    </section>
  );
}
