"use client";

import { useTranslations } from "next-intl";

import { type Path, type Suggestion, checkTemplate, pathText, scopeAt } from "@/entities/workflow";
import { FormField, TemplateInput, type TemplateSuggestion } from "@/shared/ui/kit";

import { useEditor } from "../../model/editor-context";

export function fieldId(path: Path, field: string) {
  return `${pathText(path)}-${field}`.replace(/[^a-z0-9_-]/gi, "-");
}

export function useTranslatedSuggestions() {
  const help = useTranslations("workflowHelp");
  return (suggestions: Suggestion[]): TemplateSuggestion[] =>
    suggestions.map((suggestion) => ({
      value: suggestion.value,
      group: suggestion.group,
      description: help.has(suggestion.description.key as "suggestions.input") ? help(suggestion.description.key as "suggestions.input", suggestion.description.params) : undefined,
      example: suggestion.example,
      warning: suggestion.warning ? help(suggestion.warning as "suggestions.secretUnset") : undefined,
    }));
}

export function useFilterSuggestions(path: Path, field: string) {
  const editor = useEditor();
  const help = useTranslations("workflowHelp.filters");
  const suggestions = useTranslations("workflowHelp.suggestions");
  return (subject: string, chain: string): TemplateSuggestion[] =>
    editor.filtersFor(path, field, subject, chain).map((offer) => ({
      value: offer.insert,
      label: offer.label,
      example: offer.example,
      description: [offer.element ? suggestions("forEachElement") : null, help.has(`${offer.name}.description` as "upper.description") ? help(`${offer.name}.description` as "upper.description") : null]
        .filter(Boolean)
        .join(" · "),
    }));
}

export function useGroupLabel() {
  const help = useTranslations("workflowHelp.suggestions.groups");
  return (group: string) => (help.has(group as "inputs") ? help(group as "inputs") : group);
}

export function useTemplateProblems() {
  const editor = useEditor();
  const help = useTranslations("workflowHelp.reasons");
  return (path: Path, field: string, text: string) => {
    const scope = scopeAt(editor.draft.steps, path, editor.draft.inputs.filter(Boolean), field);
    return checkTemplate(text, scope, editor.sources.portal, editor.sources.eventKnowledge, editor.catalogue.filters).map((problem) => ({ start: problem.start, end: problem.end, message: help(problem.reason, problem.params) }));
  };
}

export type StepTemplateInputProps = { path: Path; field: string; label: string; value: string; onChange: (value: string) => void; multiline?: boolean };

export function StepTemplateInput({ path, field, label, value, onChange, multiline = false }: StepTemplateInputProps) {
  const editor = useEditor();
  const translate = useTranslatedSuggestions();
  const groupLabel = useGroupLabel();
  const problemsOf = useTemplateProblems();
  const problems = problemsOf(path, field, value);
  const filterSuggestions = useFilterSuggestions(path, field);
  return (
    <TemplateInput
      id={fieldId(path, field)}
      aria-label={label}
      aria-invalid={problems.length > 0 || editor.problems.some((problem) => problem.at === `${pathText(path)}.${field}` && problem.severity === "error") ? true : undefined}
      value={value}
      onChange={onChange}
      multiline={multiline}
      suggestions={translate(editor.suggestionsFor(path, field))}
      filterSuggestions={filterSuggestions}
      problems={problems}
      groupLabel={groupLabel}
    />
  );
}

export type TemplateFieldProps = StepTemplateInputProps & { hint?: string; optional?: boolean };

export function TemplateField({ hint, optional = false, ...input }: TemplateFieldProps) {
  const editor = useEditor();
  const problemsOf = useTemplateProblems();
  const at = `${pathText(input.path)}.${input.field}`;
  const inline = problemsOf(input.path, input.field, input.value)[0]?.message;
  const recorded = editor.problems.find((problem) => problem.at === at && problem.severity === "error");
  const help = useTranslations();
  const message = inline ?? (recorded ? (recorded.text ?? help(recorded.key as "validation.required", recorded.params)) : undefined);
  return (
    <FormField id={fieldId(input.path, input.field)} label={input.label} hint={hint} optional={optional} error={message}>
      <StepTemplateInput {...input} />
    </FormField>
  );
}
