"use client";

import { useTranslations } from "next-intl";

import {
  type KindField,
  type Path,
  type Step,
  headerValues,
  pathText,
  scopeAt,
  scriptValues,
  serviceValues,
  variableValues,
  workflowValues,
} from "@/entities/workflow";
import { FormField, TemplateInput, type TemplateSuggestion } from "@/shared/ui/kit";

import { useEditor } from "../../model/editor-context";
import { fieldId, useFilterSuggestions, useGroupLabel, useTemplateProblems, useTranslatedSuggestions } from "./template-field";

export const VALUE_FIELDS = ["service", "script", "workflow", "variable", "channel", "automation"];

export function useValueSuggestions() {
  const editor = useEditor();
  const t = useTranslations("workflowEditor.values");
  return (step: Step, path: Path, field: KindField): TemplateSuggestion[] => {
    const group = t(`groups.${field.name}` as "groups.service");
    const plain = (values: { value: string; label: string; detail?: string; disabled?: boolean }[]) =>
      values.map((value) => ({ value: value.value, label: value.label === value.value ? undefined : `${value.label} · ${value.value}`, description: value.detail, disabled: value.disabled, group }));
    switch (field.name) {
      case "service":
        return plain(serviceValues(editor.sources.services));
      case "script":
        return plain(scriptValues(editor.sources.scripts));
      case "workflow":
        return plain(workflowValues(editor.sources.workflows)).map((suggestion) => ({
          ...suggestion,
          description: suggestion.description ? t("inputsOf", { inputs: suggestion.description }) : t("noInputs"),
        }));
      case "channel":
        return editor.sources.channels.map((channel) => ({
          value: channel.name,
          description: channel.readiness === "ready" ? undefined : t(`readiness.${channel.readiness}` as "readiness.missing"),
          disabled: channel.readiness !== "ready",
          group,
        }));
      case "automation":
        return editor.sources.automations.map((automation) => ({
          value: automation.id,
          label: `${automation.title} · ${automation.id}`,
          description: t("event", { event: automation.event }),
          disabled: !automation.enabled,
          group,
        }));
      case "variable":
        return plain(variableValues(scopeAt(editor.draft.steps, path, editor.draft.inputs, field.name).vars.filter((name) => name !== step.variable)));
      default:
        return [];
    }
  };
}

export function ValueField({ path, step, field, label, hint, error }: { path: Path; step: Step; field: KindField; label: string; hint?: string; error?: string }) {
  const editor = useEditor();
  const suggestionsOf = useValueSuggestions();
  const translate = useTranslatedSuggestions();
  const groupLabel = useGroupLabel();
  const problemsOf = useTemplateProblems();
  const filterSuggestions = useFilterSuggestions(path, field.name);
  const value = (step as Record<string, unknown>)[field.name];
  const text = typeof value === "string" ? value : "";
  const id = fieldId(path, field.name);
  return (
    <FormField id={id} label={label} hint={hint} optional={!field.required} error={error}>
      <TemplateInput
        id={id}
        aria-label={label}
        aria-invalid={error ? true : undefined}
        trigger="always"
        value={text}
        suggestions={suggestionsOf(step, path, field)}
        templateSuggestions={field.type === "template" ? translate(editor.suggestionsFor(path, field.name)) : undefined}
        filterSuggestions={field.type === "template" ? filterSuggestions : undefined}
        problems={field.name === "service" ? problemsOf(path, field.name, text) : []}
        groupLabel={groupLabel}
        onChange={(next) =>
          editor.change(
            path,
            (current) => {
              const copy: Record<string, unknown> = { ...current, [field.name]: next };
              if (next === "" && !field.required) {
                delete copy[field.name];
              }
              return copy as Step;
            },
            `${pathText(path)}.${field.name}`,
          )
        }
      />
    </FormField>
  );
}

export function headerSuggestions(): TemplateSuggestion[] {
  return headerValues().map((value) => ({ value: value.value }));
}
