"use client";

import { Minus, Plus } from "lucide-react";
import { useTranslations } from "next-intl";

import { type Condition, type KindField, type Path, type Step, emptyRow, filledTimeout, hiddenFields, idFor, idsOf, pathText, variableProblem } from "@/entities/workflow";
import { FormField } from "@/shared/ui/form-field";
import { Button, Input, Label, Switch } from "@/shared/ui/primitives";

import { useEditor } from "../../model/editor-context";
import { ConditionBuilder } from "./condition-builder";
import { ExclusiveChoice } from "./data/exclusive-choice";
import { ListField } from "./list-field";
import { ScriptArgumentsField } from "./script-arguments-field";
import { SampleField } from "./sample-field";
import { SELECT } from "./select-class";
import { TableField } from "./table-field";
import { StepTemplateInput, TemplateField, fieldId } from "./template-field";
import { TransformChain } from "./transform/transform-chain";
import { VALUE_FIELDS, ValueField, headerSuggestions } from "./value-field";

export const NESTED_TYPES = ["steps", "branches"];
export const INVERTED_FIELDS = ["script.fail_on_error"];
export const MULTILINE_FIELDS = ["text", "body"];
export const MOST_BRANCHES = 4;
export const FEWEST_BRANCHES = 2;

const UNSET = "-";

export function useFieldText(kind: string, field: KindField) {
  const t = useTranslations("workflowHelp.fields");
  const label = `${kind}.${field.name}.label` as Parameters<typeof t>[0];
  const help = `${kind}.${field.name}.help` as Parameters<typeof t>[0];
  return { label: t.has(label) ? t(label) : field.name, help: t.has(help) ? t(help) : undefined };
}

function Field({ path, step, field }: { path: Path; step: Step; field: KindField }) {
  const editor = useEditor();
  const t = useTranslations("workflowEditor");
  const translate = useTranslations();
  const text = useFieldText(step.kind, field);
  const at = `${pathText(path)}.${field.name}`;
  const id = fieldId(path, field.name);
  const value = (step as Record<string, unknown>)[field.name];
  const recorded = editor.problems.find((problem) => problem.at === at && problem.severity === "error");
  const error = recorded ? (recorded.text ?? translate(recorded.key as "validation.required", recorded.params)) : undefined;
  const bounds = field.minimum !== null ? t("between", { minimum: field.minimum, maximum: field.maximum ?? "" }) : undefined;
  const hint = [text.help, bounds].filter(Boolean).join(" · ") || undefined;
  const set = (next: unknown) =>
    editor.change(
      path,
      (current) => {
        const copy: Record<string, unknown> = { ...current, [field.name]: next };
        if (next === undefined) {
          delete copy[field.name];
        }
        return copy as Step;
      },
      at,
    );
  if (VALUE_FIELDS.includes(field.name) && (field.type === "template" || field.type === "script" || field.type === "workflow" || field.type === "name" || field.type === "channel" || field.type === "automation")) {
    return <ValueField path={path} step={step} field={field} label={text.label} hint={hint} error={error} />;
  }
  switch (field.type) {
    case "template":
    case "name":
      return (
        <TemplateField
          path={path}
          field={field.name}
          label={text.label}
          hint={hint}
          optional={!field.required}
          multiline={MULTILINE_FIELDS.includes(field.name)}
          value={typeof value === "string" ? value : ""}
          onChange={(next) => set(next === "" && !field.required ? undefined : next)}
        />
      );
    case "integer":
      if (field.templated) {
        return (
          <FormField id={id} label={text.label} hint={typeof value === "string" ? t("checkedWhenRuns") : hint} error={error} optional={!field.required}>
            <StepTemplateInput
              path={path}
              field={field.name}
              label={text.label}
              value={value === undefined || value === null ? (filledTimeout(step.kind, field.name) ? (field.default ?? "") : "") : String(value)}
              onChange={(next) => set(numberOrTemplate(next))}
            />
          </FormField>
        );
      }
      return (
        <FormField id={id} label={text.label} hint={hint} error={error} optional={!field.required}>
          <Input
            id={id}
            type="number"
            inputMode="numeric"
            placeholder={field.default ?? undefined}
            value={typeof value === "number" ? String(value) : ""}
            onChange={(change) => set(change.target.value === "" ? undefined : Number(change.target.value))}
          />
        </FormField>
      );
    case "boolean": {
      const inverted = INVERTED_FIELDS.includes(`${step.kind}.${field.name}`);
      return (
        <div className="grid gap-1">
          <div className="flex items-center gap-2">
            <Switch
              id={id}
              checked={(typeof value === "boolean" ? value : field.default === "true") !== inverted}
              onCheckedChange={(checked) => {
                const next = checked !== inverted;
                set(String(next) === field.default ? undefined : next);
              }}
            />
            <Label htmlFor={id}>{text.label}</Label>
          </div>
          {hint ? <p className="text-xs text-muted-foreground">{hint}</p> : null}
        </div>
      );
    }
    case "choice": {
      const current = typeof value === "string" && value !== "" ? value : (field.default ?? UNSET);
      return (
        <FormField id={id} label={text.label} hint={hint} error={error} optional={!field.required}>
          <select id={id} className={SELECT} value={current} onChange={(change) => set(change.target.value === UNSET ? undefined : change.target.value)}>
            {current === UNSET ? <option value={UNSET}>{t("choose")}</option> : null}
            {field.choices.map((option) => (
              <option key={option} value={option}>
                {t.has(`choices.${option}` as "choices.succeeded") ? t(`choices.${option}` as "choices.succeeded") : option}
              </option>
            ))}
          </select>
        </FormField>
      );
    }
    case "template-list":
      if (step.kind === "script" && field.name === "args") {
        const values = Array.isArray(value) ? (value as string[]) : [];
        return (
          <ScriptArgumentsField
            path={path}
            label={text.label}
            hint={hint}
            script={typeof step.script === "string" ? step.script : ""}
            values={values}
            onChange={(next) => set(next.length === 0 ? undefined : next)}
          />
        );
      }
      return <ListField path={path} field={field.name} label={text.label} hint={hint} values={Array.isArray(value) ? (value as string[]) : []} onChange={(next) => set(next.length === 0 ? undefined : next)} />;
    case "template-table": {
      const called = step.kind === "workflow" ? editor.sources.workflows.find((workflow) => workflow.id === step.workflow) : undefined;
      return (
        <TableField
          path={path}
          field={field.name}
          label={text.label}
          hint={hint}
          values={(value as Record<string, string> | undefined) ?? {}}
          fixedKeys={step.kind === "workflow" ? (called?.inputs ?? []).map((input) => input.name) : undefined}
          keySuggestions={field.name === "headers" ? headerSuggestions() : field.name === "fields" ? eventFieldSuggestions(editor, step) : []}
          templateKeys={field.template_keys}
          keyProblem={
            field.name === "env"
              ? (key) => {
                  const problem = variableProblem(key);
                  return problem ? translate(`workflowEditor.problems.${problem}`) : undefined;
                }
              : undefined
          }
          onChange={(next) => set(Object.keys(next).length === 0 ? undefined : next)}
        />
      );
    }
    case "operations":
      return <TransformChain path={path} step={step} label={text.label} hint={text.help} />;
    case "sample":
      return <SampleField path={path} step={step} label={text.label} hint={text.help} />;
    case "condition":
      return value === undefined ? (
        <FormField id={id} label={text.label} hint={hint} optional>
          <Button id={id} type="button" variant="outline" size="sm" className="w-fit" onClick={() => set(emptyRow())}>
            <Plus aria-hidden />
            {t("condition.add")}
          </Button>
        </FormField>
      ) : (
        <div className="grid gap-1">
          <ConditionBuilder path={path} field={field.name} label={text.label} condition={value as Condition} onChange={set} />
          {hint ? <p className="text-xs text-muted-foreground">{hint}</p> : null}
        </div>
      );
    default:
      return null;
  }
}

function eventFieldSuggestions(editor: ReturnType<typeof useEditor>, step: Step) {
  const event = editor.sources.automations.find((automation) => automation.id === step.automation)?.event;
  return (editor.catalogue.events.find((entry) => entry.name === event)?.fields ?? []).map((field) => ({ value: field }));
}

function Branches({ path, step }: { path: Path; step: Step }) {
  const t = useTranslations("workflowEditor");
  const editor = useEditor();
  const branches = step.branches ?? [];
  const set = (next: Step[][]) => editor.change(path, (current) => ({ ...current, branches: next }));
  return (
    <div className="flex flex-wrap items-center gap-2">
      <span className="text-sm">{t("branchCount", { count: branches.length })}</span>
      <Button type="button" variant="outline" size="sm" disabled={branches.length >= MOST_BRANCHES} onClick={() => set([...branches, []])}>
        <Plus aria-hidden />
        {t("addBranch")}
      </Button>
      <Button type="button" variant="outline" size="sm" disabled={branches.length <= FEWEST_BRANCHES} onClick={() => set(branches.slice(0, -1))}>
        <Minus aria-hidden />
        {t("removeBranch", { number: branches.length })}
      </Button>
    </div>
  );
}

export function StepForm({ path, step }: { path: Path; step: Step }) {
  const editor = useEditor();
  const t = useTranslations("workflowEditor");
  const translate = useTranslations();
  const kind = editor.kindOf(step.kind);
  const at = pathText(path);
  const id = at.replace(/[^a-z0-9_-]/gi, "-");
  const idProblem = editor.problems.find((problem) => problem.at === `${at}.id`);
  const hidden = hiddenFields(step, kind);
  const fields = (kind?.fields ?? []).filter((field) => !NESTED_TYPES.includes(field.type) && !hidden.has(field.name));
  return (
    <div className="grid gap-4">
      <div className="grid gap-4 sm:grid-cols-2">
        <FormField id={`${id}-label`} label={t("label")} optional>
          <Input
            id={`${id}-label`}
            value={step.label ?? ""}
            onChange={(change) =>
              editor.change(
                path,
                (current) => {
                  const others = idsOf(editor.draft.steps);
                  others.delete(current.id);
                  const follows = current.id === idFor(current.label ?? current.kind, others);
                  const label = change.target.value || undefined;
                  return { ...current, label, id: follows ? idFor(label ?? current.kind, others) : current.id };
                },
                `${at}.label`,
              )
            }
          />
        </FormField>
        <FormField
          id={`${id}-id`}
          label={t("stepId")}
          hint={t("stepIdHint")}
          error={idProblem ? (idProblem.text ?? translate(idProblem.key as "validation.required", idProblem.params)) : undefined}
        >
          <Input
            id={`${id}-id`}
            className="font-mono"
            spellCheck={false}
            autoComplete="off"
            value={step.id}
            onChange={(change) => editor.change(path, (current) => ({ ...current, id: change.target.value }), `${at}.id`)}
          />
        </FormField>
      </div>
      {step.kind === "parallel" ? <Branches path={path} step={step} /> : null}
      {(kind?.exclusive ?? []).map((group) => (
        <ExclusiveChoice key={group.join("-")} step={step} group={group} onChange={(next) => editor.change(path, () => next, `${at}.${group[0]}-choice`)} />
      ))}
      <div className="grid gap-4">
        {fields.map((field) => (
          <Field key={field.name} path={path} step={step} field={field} />
        ))}
      </div>
    </div>
  );
}

export function numberOrTemplate(text: string): number | string | undefined {
  const trimmed = text.trim();
  if (trimmed === "") {
    return undefined;
  }
  return /^-?\d+$/.test(trimmed) ? Number(trimmed) : text;
}
