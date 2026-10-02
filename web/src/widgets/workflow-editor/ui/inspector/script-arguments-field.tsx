"use client";

import { useState } from "react";

import { ArgumentMode, DeclaredArguments, HeaderProblems, fromArgs, toArgs } from "@/entities/script";
import type { Path } from "@/entities/workflow";
import { Label } from "@/shared/ui/kit";

import { useEditor } from "../../model/editor-context";
import { ListField } from "./list-field";
import { StepTemplateInput, fieldId } from "./template-field";

export type ScriptArgumentsFieldProps = {
  path: Path;
  label: string;
  hint?: string;
  script: string;
  values: string[];
  onChange: (values: string[]) => void;
};

export function ScriptArgumentsField({ path, label, hint, script, values, onChange }: ScriptArgumentsFieldProps) {
  const editor = useEditor();
  const [asList, setAsList] = useState(false);
  const entry = editor.sources.scripts.find((candidate) => candidate.path === script);
  const declared = entry?.arguments ?? [];
  const fitted = declared.length > 0 ? fromArgs(declared, values) : null;
  const asFields = fitted !== null && !asList;
  return (
    <div className="grid gap-2">
      {entry?.description ? <p className="text-xs text-muted-foreground">{entry.description}</p> : null}
      <HeaderProblems problems={entry?.argument_problems ?? []} />
      {declared.length > 0 ? <ArgumentMode asList={!asFields} fits={fitted !== null} onChange={setAsList} /> : null}
      {asFields ? (
        <div className="grid gap-2">
          <Label>{label}</Label>
          <DeclaredArguments
            idPrefix={fieldId(path, "args")}
            declared={declared}
            values={fitted}
            onChange={(next) => onChange(toArgs(declared, next))}
            renderText={(argument, props) => (
              <StepTemplateInput
                path={path}
                field={`args-${argument.name.replace(/^-+/, "")}`}
                label={props.label}
                value={props.value}
                onChange={props.onChange}
              />
            )}
          />
        </div>
      ) : (
        <ListField path={path} field="args" label={label} hint={hint} values={values} onChange={onChange} />
      )}
    </div>
  );
}
