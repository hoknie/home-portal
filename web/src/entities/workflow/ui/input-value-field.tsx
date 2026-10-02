"use client";

import { Plus, Trash2 } from "lucide-react";
import { useTranslations } from "next-intl";

import { Button, Input, Switch } from "@/shared/ui/kit";

import type { InputType } from "../model/schema";

export type InputValueFieldProps = { id: string; label: string; type: InputType; value: unknown; placeholder?: string; onChange: (value: unknown) => void };

function Rows({ id, label, rows, onChange, keyed }: { id: string; label: string; rows: [string, string][]; onChange: (rows: [string, string][]) => void; keyed: boolean }) {
  const t = useTranslations("workflowInputs");
  const update = (index: number, row: [string, string]) => onChange(rows.map((current, position) => (position === index ? row : current)));
  return (
    <div className="grid gap-1.5" role="group" aria-label={label}>
      {rows.map(([key, text], index) => (
        <div key={`${id}-${index}`} className="flex gap-1">
          {keyed ? (
            <Input aria-label={t("key", { number: index + 1 })} className="w-32 font-mono" value={key} onChange={(change) => update(index, [change.target.value, text])} />
          ) : null}
          <Input aria-label={t("item", { number: index + 1 })} value={text} onChange={(change) => update(index, [key, change.target.value])} />
          <Button type="button" variant="ghost" size="icon" aria-label={t("removeRow", { number: index + 1 })} onClick={() => onChange(rows.filter((_, position) => position !== index))}>
            <Trash2 aria-hidden />
          </Button>
        </div>
      ))}
      <Button type="button" variant="outline" size="sm" className="w-fit" onClick={() => onChange([...rows, ["", ""]])}>
        <Plus aria-hidden />
        {t(keyed ? "addKey" : "addItem")}
      </Button>
    </div>
  );
}

function itemOf(text: string): unknown {
  const trimmed = text.trim();
  if (trimmed !== "" && /^(-?\d+(\.\d+)?|true|false)$/.test(trimmed)) {
    return JSON.parse(trimmed) as unknown;
  }
  return text;
}

function textOfItem(value: unknown): string {
  return typeof value === "string" ? value : JSON.stringify(value ?? "");
}

export function InputValueField({ id, label, type, value, placeholder, onChange }: InputValueFieldProps) {
  switch (type) {
    case "number":
      return (
        <Input
          id={id}
          aria-label={label}
          type="number"
          placeholder={placeholder}
          value={typeof value === "number" ? String(value) : ""}
          onChange={(change) => onChange(change.target.value === "" ? null : Number(change.target.value))}
        />
      );
    case "boolean":
      return <Switch id={id} aria-label={label} checked={value === true} onCheckedChange={onChange} />;
    case "list":
      return (
        <Rows
          id={id}
          label={label}
          keyed={false}
          rows={(Array.isArray(value) ? value : []).map((item) => ["", textOfItem(item)])}
          onChange={(rows) => onChange(rows.map(([, text]) => itemOf(text)))}
        />
      );
    case "object": {
      const entries = value !== null && typeof value === "object" && !Array.isArray(value) ? Object.entries(value as Record<string, unknown>) : [];
      return (
        <Rows
          id={id}
          label={label}
          keyed
          rows={entries.map(([key, item]) => [key, textOfItem(item)])}
          onChange={(rows) => onChange(Object.fromEntries(rows.map(([key, text]) => [key, itemOf(text)])))}
        />
      );
    }
    default:
      return <Input id={id} aria-label={label} placeholder={placeholder} value={typeof value === "string" ? value : ""} onChange={(change) => onChange(change.target.value)} />;
  }
}
