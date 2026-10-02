"use client";

import { Plus, Trash2 } from "lucide-react";
import { useTranslations } from "next-intl";

import { END_OF_WORKFLOW, type WorkflowOutput } from "@/entities/workflow";
import { Button, Input, Label } from "@/shared/ui/primitives";
import { TemplateInput } from "@/shared/ui/template-input";

import { useEditor } from "../../../model/editor-context";
import { useFilterSuggestions, useGroupLabel, useTranslatedSuggestions } from "../template-field";

export type OutputsEditorProps = { outputs: WorkflowOutput[]; problemAt: (at: string) => string | undefined; onChange: (key: string, outputs: WorkflowOutput[]) => void };

const VALUE = "value";

export function OutputsEditor({ outputs, problemAt, onChange }: OutputsEditorProps) {
  const t = useTranslations("workflowEditor");
  const editor = useEditor();
  const translate = useTranslatedSuggestions();
  const groupLabel = useGroupLabel();
  const filterSuggestions = useFilterSuggestions(END_OF_WORKFLOW, VALUE);
  const suggestions = translate(editor.suggestionsFor(END_OF_WORKFLOW, VALUE).filter((suggestion) => suggestion.group !== "event" && suggestion.group !== "loop"));
  const change = (index: number, key: string, next: Partial<WorkflowOutput>) =>
    onChange(
      `outputs[${index}].${key}`,
      outputs.map((current, position) => (position === index ? { ...current, ...next } : current)),
    );
  return (
    <div className="grid gap-2" data-outputs="">
      <Label>{t("outputs")}</Label>
      <p className="text-xs text-muted-foreground">{t("outputsHint")}</p>
      {outputs.map((output, index) => {
        const problem = problemAt(`outputs[${index}].name`) ?? problemAt(`outputs[${index}].value`) ?? problemAt(`outputs[${index}].description`);
        return (
          <fieldset key={`output-${index}`} className="grid gap-1.5 rounded-lg border border-glass-edge p-2" aria-label={t("outputName", { number: index + 1 })}>
            <div className="flex gap-1">
              <Input
                aria-label={t("outputName", { number: index + 1 })}
                aria-invalid={problem ? true : undefined}
                className="font-mono"
                placeholder={t("outputNamePlaceholder")}
                value={output.name}
                onChange={(event) => change(index, "name", { name: event.target.value })}
              />
              <Button type="button" variant="ghost" size="icon" aria-label={t("removeOutput", { name: output.name })} onClick={() => onChange("outputs", outputs.filter((_, position) => position !== index))}>
                <Trash2 aria-hidden />
              </Button>
            </div>
            <TemplateInput
              id={`workflow-output-value-${index}`}
              aria-label={t("outputValue", { number: index + 1 })}
              value={output.value}
              onChange={(value) => change(index, "value", { value })}
              suggestions={suggestions}
              filterSuggestions={filterSuggestions}
              groupLabel={groupLabel}
            />
            <Input
              aria-label={t("outputDescription", { number: index + 1 })}
              placeholder={t("outputDescriptionPlaceholder")}
              value={output.description ?? ""}
              onChange={(event) => change(index, "description", { description: event.target.value === "" ? null : event.target.value })}
            />
            {problem ? (
              <p role="alert" className="text-sm text-destructive">
                {problem}
              </p>
            ) : null}
          </fieldset>
        );
      })}
      <Button type="button" variant="outline" size="sm" className="w-fit" onClick={() => onChange("outputs", [...outputs, { name: "", value: "", description: null }])}>
        <Plus aria-hidden />
        {t("addOutput")}
      </Button>
    </div>
  );
}
