"use client";

import { useTranslations } from "next-intl";
import { useState } from "react";

import type { Scripts } from "@/entities/automation";
import { ArgumentMode, DeclaredArguments, HeaderProblems, fromArgs, toArgs } from "@/entities/script";
import { type Workflow, InputValueField } from "@/entities/workflow";
import { FormField, Input, TagInput } from "@/shared/ui/kit";

type ScriptEntry = Scripts["scripts"][number];

export type ScriptArgumentsProps = { script: ScriptEntry | undefined; args: string[]; onChange: (args: string[]) => void };

export function ScriptArguments({ script, args, onChange }: ScriptArgumentsProps) {
  const t = useTranslations("layoutEditor.data");
  const [asList, setAsList] = useState(false);
  const declared = script?.arguments ?? [];
  const fitted = declared.length > 0 ? fromArgs(declared, args) : null;
  const asFields = fitted !== null && !asList;
  return (
    <div className="grid gap-3" data-source-arguments="">
      <HeaderProblems problems={script?.argument_problems ?? []} />
      {declared.length > 0 ? <ArgumentMode asList={!asFields} fits={fitted !== null} onChange={setAsList} /> : null}
      {asFields ? (
        <DeclaredArguments
          idPrefix="source-arg"
          declared={declared}
          values={fitted}
          onChange={(values) => onChange(toArgs(declared, values))}
          renderText={(_, props) => <Input id={props.id} value={props.value} placeholder={props.placeholder} onChange={(event) => props.onChange(event.target.value)} />}
        />
      ) : (
        <FormField id="source-args" label={t("args")} optional>
          <TagInput
            id="source-args"
            values={args}
            suggestions={[]}
            onChange={onChange}
            removeLabel={(value) => t("removeArgument", { value })}
            createLabel={(value) => t("addArgument", { value })}
          />
        </FormField>
      )}
    </div>
  );
}

export type WorkflowInputsProps = {
  workflow: Workflow | undefined;
  values: Record<string, unknown>;
  errors: Record<string, string>;
  onChange: (values: Record<string, unknown>) => void;
};

function shown(value: unknown) {
  return value === null || value === undefined ? undefined : typeof value === "string" ? value : JSON.stringify(value);
}

export function WorkflowInputs({ workflow, values, errors, onChange }: WorkflowInputsProps) {
  const t = useTranslations("layoutEditor.data");
  return (
    <div className="grid gap-3" data-source-inputs="">
      {(workflow?.inputs ?? []).map((input) => {
        const id = `source-input-${input.name}`;
        const fallback = shown(input.default);
        const hint = [input.description, fallback === undefined ? null : t("inputDefault", { value: fallback })].filter(Boolean).join(" · ");
        return (
          <FormField key={input.name} id={id} label={input.name} hint={hint || undefined} error={errors[`source.inputs.${input.name}`]} optional>
            <InputValueField
              id={id}
              label={input.name}
              type={input.type}
              value={values[input.name]}
              placeholder={fallback}
              onChange={(value) => {
                const next = { ...values };
                if (value === null || value === undefined || value === "" || (typeof value === "number" && Number.isNaN(value))) {
                  delete next[input.name];
                } else {
                  next[input.name] = value;
                }
                onChange(next);
              }}
            />
          </FormField>
        );
      })}
    </div>
  );
}
