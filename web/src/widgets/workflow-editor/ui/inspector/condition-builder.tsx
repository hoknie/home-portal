"use client";

import { Plus, Trash2 } from "lucide-react";
import { useTranslations } from "next-intl";

import { type Condition, DEEPEST_CONDITION, type Join, JOINS, type Path, emptyRow, joinOf, joined, knownValues, operatorName, rowsOf, takesRight } from "@/entities/workflow";
import { BareButton, Button, NativeSelect } from "@/shared/ui/kit";

import { useEditor } from "../../model/editor-context";
import { StepTemplateInput } from "./template-field";

export type ConditionBuilderProps = { path: Path; field: string; label: string; condition: Condition; onChange: (condition: Condition) => void; depth?: number };

function Row({ path, field, row, onChange }: { path: Path; field: string; row: Condition; onChange: (row: Condition) => void }) {
  const t = useTranslations("workflowEditor.condition");
  const editor = useEditor();
  const right = takesRight(row.op, editor.catalogue);
  const known = right ? knownValues(row.left ?? "", editor.sources.states) : [];
  return (
    <div className="grid min-w-0 gap-1.5">
      <StepTemplateInput path={path} field={`${field}.left`} label={t("left")} value={row.left ?? ""} onChange={(left) => onChange({ ...row, left })} />
      <NativeSelect
        aria-label={t("operator")}
        className="font-mono text-xs"
        value={row.op ?? "=="}
        onChange={(change) => {
          const op = change.target.value;
          onChange(takesRight(op, editor.catalogue) ? { ...row, op, right: row.right ?? "" } : { left: row.left, op });
        }}
      >
        {editor.catalogue.operators.map((operator) => (
          <option key={operator.name} value={operator.name}>
            {t.has(`operators.${operatorName(operator.name)}` as "operators.equals") ? t(`operators.${operatorName(operator.name)}` as "operators.equals") : operator.name}
          </option>
        ))}
      </NativeSelect>
      {right ? <StepTemplateInput path={path} field={`${field}.right`} label={t("right")} value={row.right ?? ""} onChange={(value) => onChange({ ...row, right: value })} /> : null}
      {known.length > 0 ? (
        <div className="flex flex-wrap items-center gap-1" role="group" aria-label={t("knownValues")}>
          <span className="text-xs text-muted-foreground">{t("knownValues")}</span>
          {known.map((value) => (
            <BareButton
              key={value.value}
              className="rounded-full border border-glass-edge bg-glass-tint px-2 py-0.5 font-mono text-xs hover:bg-accent"
              onClick={() => onChange({ ...row, right: value.value })}
            >
              {value.label}
            </BareButton>
          ))}
        </div>
      ) : null}
    </div>
  );
}

export function ConditionBuilder({ path, field, label, condition, onChange, depth = 1 }: ConditionBuilderProps) {
  const t = useTranslations("workflowEditor.condition");
  const join: Join = joinOf(condition) ?? "all";
  const rows = rowsOf(condition);
  const combine = (how: Join, list: Condition[]) => (depth > 1 ? { [how]: list } : joined(how, list));
  const replaceRow = (index: number, row: Condition) => onChange(combine(join, rows.map((current, position) => (position === index ? row : current))));
  const removeRow = (index: number) => onChange(combine(join, rows.filter((_, position) => position !== index)));
  const grouped = joinOf(condition) !== null;
  return (
    <fieldset className="grid min-w-0 gap-2 rounded-lg border border-glass-edge p-3">
      <legend className="px-1 text-sm font-medium">{label}</legend>
      <div className="flex flex-wrap items-center gap-2 text-sm">
        <span>{t("match")}</span>
        <NativeSelect aria-label={t("join")} className="w-40" value={join} onChange={(change) => onChange(combine(change.target.value as Join, rows))}>
          {JOINS.map((name) => (
            <option key={name} value={name}>
              {t(`joins.${name}`)}
            </option>
          ))}
        </NativeSelect>
      </div>
      {rows.map((row, index) => {
        const at = grouped ? `${field}.${join}[${index}]` : field;
        return (
          <div key={`${depth}-${index}`} className="flex min-w-0 items-start gap-1 border-s-2 border-glass-edge ps-2">
            <div className="min-w-0 flex-1">
              {joinOf(row) === null ? (
                <Row path={path} field={at} row={row} onChange={(changed) => replaceRow(index, changed)} />
              ) : (
                <ConditionBuilder path={path} field={at} label={t("group")} condition={row} depth={depth + 1} onChange={(changed) => replaceRow(index, changed)} />
              )}
            </div>
            <Button type="button" variant="ghost" size="icon" aria-label={t("removeRow")} disabled={rows.length === 1} onClick={() => removeRow(index)}>
              <Trash2 aria-hidden />
            </Button>
          </div>
        );
      })}
      <div className="flex flex-wrap gap-2">
        <Button type="button" variant="outline" size="sm" onClick={() => onChange({ [join]: [...rows, emptyRow()] })}>
          <Plus aria-hidden />
          {t("addRow")}
        </Button>
        {depth < DEEPEST_CONDITION ? (
          <Button type="button" variant="outline" size="sm" onClick={() => onChange({ [join]: [...rows, { any: [emptyRow()] }] })}>
            <Plus aria-hidden />
            {t("addGroup")}
          </Button>
        ) : null}
      </div>
    </fieldset>
  );
}
