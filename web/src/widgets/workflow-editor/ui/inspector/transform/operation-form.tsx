"use client";

import { useTranslations } from "next-intl";

import { type Condition, FILTERS, type Operation, type Path, emptyRow, jsonKeys } from "@/entities/workflow";
import { FormField } from "@/shared/ui/form-field";
import { Input } from "@/shared/ui/primitives";
import { TemplateInput } from "@/shared/ui/template-input";

import { ConditionBuilder } from "../condition-builder";
import { SELECT } from "../select-class";
import type { NestedChain } from "./transform-chain";
import { StepTemplateInput, fieldId } from "../template-field";

export type OperationFormProps = {
  path: Path;
  field: string;
  depth: number;
  operation: Operation;
  item: unknown;
  from: "sample" | "run" | null;
  nested: NestedChain;
  onChange: (operation: Operation) => void;
};

function literalOf(text: string, type: string): unknown {
  if (type === "number") {
    return text.trim() === "" ? undefined : Number(text);
  }
  if (type === "any") {
    try {
      const parsed: unknown = JSON.parse(text);
      return typeof parsed === "object" && parsed !== null ? text : parsed;
    } catch {
      return text;
    }
  }
  return text;
}

function KeyField({ id, label, value, keys, onChange }: { id: string; label: string; value: string; keys: string[]; onChange: (key: string) => void }) {
  return (
    <FormField id={id} label={label}>
      <TemplateInput id={id} aria-label={label} trigger="always" value={value} suggestions={keys.map((key) => ({ value: key }))} onChange={onChange} />
    </FormField>
  );
}

export function OperationForm({ path, field, depth, operation, item, from, nested: Nested, onChange }: OperationFormProps) {
  const t = useTranslations("workflowEditor.transform");
  const id = fieldId(path, field);
  const keys = jsonKeys(item === undefined ? null : JSON.stringify(item)).map((key) => key.path);
  switch (operation.op) {
    case "filter":
      return (
        <ConditionBuilder
          path={path}
          field={`${field}.where`}
          label={t("where")}
          condition={operation.where ?? emptyRow()}
          onChange={(where: Condition) => onChange({ ...operation, where })}
        />
      );
    case "map":
      return (
        <FormField id={fieldId(path, `${field}.to`)} label={t("to")} hint={t("toHint")}>
          <StepTemplateInput path={path} field={`${field}.to`} label={t("to")} value={operation.to ?? ""} onChange={(to) => onChange({ ...operation, to })} />
        </FormField>
      );
    case "sort_by":
      return (
        <div className="grid gap-3 sm:grid-cols-[1fr_auto]">
          <KeyField id={`${id}-key`} label={t("key")} value={operation.key ?? ""} keys={keys} onChange={(key) => onChange({ ...operation, key })} />
          <FormField id={`${id}-order`} label={t("order")}>
            <select id={`${id}-order`} className={SELECT} value={operation.order ?? "asc"} onChange={(event) => onChange({ ...operation, order: event.target.value })}>
              <option value="asc">{t("orders.asc")}</option>
              <option value="desc">{t("orders.desc")}</option>
            </select>
          </FormField>
        </div>
      );
    case "each":
      return (
        <div className="grid gap-2 border-s-2 border-primary/30 ps-3">
          <p className="text-xs text-muted-foreground">{t("eachHint")}</p>
          <Nested
            path={path}
            field={`${field}.operations`}
            operations={operation.operations ?? []}
            input={item === undefined ? null : { value: item, from: from ?? "sample" }}
            depth={depth + 1}
            onChange={(operations) => onChange({ ...operation, operations })}
          />
        </div>
      );
    case "group_by":
    case "count_by":
      return <KeyField id={`${id}-key`} label={t("key")} value={operation.key ?? ""} keys={keys} onChange={(key) => onChange({ ...operation, key })} />;
    default: {
      const filter = FILTERS[operation.op];
      if (!filter || filter.arguments.length === 0) {
        return null;
      }
      const args = operation.args ?? [];
      return (
        <div className="grid gap-3 sm:grid-cols-2">
          {filter.arguments.map((argument, position) => {
            const argumentId = `${id}-${argument.name}`;
            const current = args[position];
            const set = (text: string) => {
              const next = [...args];
              next[position] = literalOf(text, argument.type);
              while (next.length > 0 && next.at(-1) === undefined) {
                next.pop();
              }
              onChange({ ...operation, args: next });
            };
            if (argument.name === "key") {
              return <KeyField key={argument.name} id={argumentId} label={argument.name} value={typeof current === "string" ? current : ""} keys={keys} onChange={set} />;
            }
            return (
              <FormField key={argument.name} id={argumentId} label={argument.name} optional={!argument.required}>
                <Input
                  id={argumentId}
                  type={argument.type === "number" ? "number" : "text"}
                  spellCheck={false}
                  value={current === undefined || current === null ? "" : typeof current === "string" ? current : JSON.stringify(current)}
                  onChange={(event) => set(event.target.value)}
                />
              </FormField>
            );
          })}
        </div>
      );
    }
  }
}
