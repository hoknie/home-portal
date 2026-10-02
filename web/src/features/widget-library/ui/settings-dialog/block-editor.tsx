"use client";

import { Trash2 } from "lucide-react";
import { useTranslations } from "next-intl";

import { Button, Checkbox, Input, NativeSelect } from "@/shared/ui/kit";

import { type RawBlock, TONED, replaced, without } from "../../model/blocks";
import { ActionEditor } from "./action-editor";
import { TemplateField } from "./template-field";
import { ToneEditor } from "./tone-editor";

export type BlockEditorProps = { id: string; block: RawBlock; errors: Record<string, string>; onChange: (block: RawBlock) => void };

type Row = Record<string, string>;

const LIST_KINDS: readonly string[] = ["list", "table"];

export function BlockEditor({ id, block, errors, onChange }: BlockEditorProps) {
  const t = useTranslations("layoutEditor.blocks");
  const string = (key: string) => (typeof block[key] === "string" ? (block[key] as string) : "");
  const set = (key: string, value: unknown) => onChange({ ...block, [key]: value === "" ? undefined : value });
  const field = (key: string, options: { optional?: boolean; insideList?: boolean; multiline?: boolean } = {}) => (
    <TemplateField
      id={`${id}-${key}`}
      label={t(`fields.${key}` as "fields.label")}
      value={string(key)}
      error={errors[key]}
      optional={options.optional}
      items={options.insideList ? string("items") : null}
      field={key}
      hint={t.has(`hints.${key}` as "hints.label") ? t(`hints.${key}` as "hints.label") : undefined}
      example={t.has(`examples.${key}` as "examples.label") ? t(`examples.${key}` as "examples.label") : undefined}
      multiline={options.multiline}
      onChange={(value) => set(key, value)}
    />
  );
  const choose = (key: string, values: readonly string[], fallback: string, labels: (value: string) => string) => (
    <label className="grid gap-1 text-sm">
      {t(`fields.${key}` as "fields.size")}
      <NativeSelect value={string(key) || fallback} onChange={(event) => set(key, event.target.value)}>
        {values.map((value) => (
          <option key={value} value={value}>
            {labels(value)}
          </option>
        ))}
      </NativeSelect>
    </label>
  );
  const limit = (
    <label className="grid gap-1 text-sm">
      {t("fields.limit")}
      <Input type="number" min={1} max={50} placeholder="10" value={typeof block.limit === "number" ? block.limit : ""} onChange={(event) => set("limit", event.target.value === "" ? "" : Number(event.target.value))} />
    </label>
  );
  const rows = (key: "columns" | "pairs", names: [string, string], fresh: Row) => {
    const list = (Array.isArray(block[key]) ? block[key] : []) as Row[];
    return (
      <fieldset className="grid gap-2">
        <legend className="text-sm font-medium">{t(`fields.${key}`)}</legend>
        {errors[key] ? <p role="alert" className="text-xs text-destructive">{errors[key]}</p> : null}
        {list.map((row, index) => (
          <div key={index} className="grid items-end gap-2 @sm:grid-cols-[1fr_1fr_auto]">
            {names.map((name) => (
              <TemplateField
                key={name}
                id={`${id}-${key}-${index}-${name}`}
                label={t(`fields.${name}` as "fields.key")}
                value={row[name] ?? ""}
                error={errors[`${key}[${index}].${name}`]}
                items={key === "columns" && name === "value" ? string("items") : null}
                onChange={(value) => set(key, replaced(list, index, { ...row, [name]: value }))}
              />
            ))}
            <Button type="button" variant="ghost" size="icon" aria-label={t("removeRow")} onClick={() => set(key, without(list, index))}>
              <Trash2 aria-hidden />
            </Button>
          </div>
        ))}
        <Button type="button" variant="ghost" size="sm" className="w-fit" onClick={() => set(key, [...list, fresh])}>
          {key === "columns" ? t("addColumn") : t("addPair")}
        </Button>
      </fieldset>
    );
  };
  return (
    <div className="@container grid gap-3">
      {block.kind === "stat" ? (
        <>
          {field("label")}
          {field("value")}
          <div className="grid gap-3 @sm:grid-cols-2">
            {field("unit", { optional: true })}
            <label className="grid gap-1 text-sm">
              {t("fields.icon")}
              <Input value={string("icon")} onChange={(event) => set("icon", event.target.value)} />
            </label>
          </div>
          {field("caption", { optional: true })}
        </>
      ) : null}
      {block.kind === "text" ? (
        <>
          {field("text", { multiline: true })}
          <div className="grid gap-3 @md:grid-cols-3">
            {choose("size", ["small", "normal", "large"], "normal", (value) => t(`sizes.${value as "small"}`))}
            {choose("weight", ["normal", "strong"], "normal", (value) => t(`weights.${value as "normal"}`))}
            <label className="flex items-center gap-2 self-end text-sm">
              <Checkbox checked={block.muted === true} onCheckedChange={(checked) => set("muted", checked === true || "")} />
              {t("fields.muted")}
            </label>
          </div>
        </>
      ) : null}
      {block.kind === "markdown" ? field("text", { multiline: true }) : null}
      {block.kind === "badge" ? field("text") : null}
      {LIST_KINDS.includes(block.kind) ? field("items") : null}
      {block.kind === "list" ? (
        <>
          {field("text", { insideList: true })}
          {field("secondary", { optional: true, insideList: true })}
        </>
      ) : null}
      {block.kind === "table" ? rows("columns", ["header", "value"], { header: "", value: "{{item}}" }) : null}
      {LIST_KINDS.includes(block.kind) ? (
        <div className="grid gap-3 @sm:grid-cols-2">
          {limit}
          {field("empty", { optional: true })}
        </div>
      ) : null}
      {block.kind === "key-values" ? rows("pairs", ["key", "value"], { key: "", value: "" }) : null}
      {block.kind === "progress" ? (
        <>
          {field("label", { optional: true })}
          <div className="grid gap-3 @sm:grid-cols-2">
            {field("value")}
            <TemplateField
              id={`${id}-maximum`}
              label={t("fields.maximum")}
              value={typeof block.maximum === "number" ? String(block.maximum) : string("maximum")}
              error={errors.maximum}
              optional
              onChange={(value) => set("maximum", /^\d+(\.\d+)?$/.test(value) ? Number(value) : value)}
            />
          </div>
          {field("caption", { optional: true })}
        </>
      ) : null}
      {block.kind === "button" ? (
        <>
          <div className="grid gap-3 @sm:grid-cols-2">
            {field("label")}
            {choose("style", ["primary", "secondary", "ghost"], "secondary", (value) => t(`styles.${value as "primary"}`))}
          </div>
          <label className="grid gap-1 text-sm">
            {t("fields.icon")}
            <Input value={string("icon")} onChange={(event) => set("icon", event.target.value)} />
          </label>
          {field("confirm", { optional: true })}
          <ActionEditor
            id={`${id}-action`}
            action={(block.action ?? {}) as Record<string, never>}
            errors={Object.fromEntries(Object.entries(errors).filter(([key]) => key === "action" || key.startsWith("action.")).map(([key, message]) => [key === "action" ? "" : key.slice("action.".length), message]))}
            onChange={(action) => set("action", action)}
          />
        </>
      ) : null}
      {block.kind === "row" || block.kind === "column" ? choose("gap", ["small", "normal"], "normal", (value) => t(`gaps.${value as "small"}`)) : null}
      {TONED.includes(block.kind) ? <ToneEditor id={id} block={block} onChange={onChange} /> : null}
      {errors.tone ? <p role="alert" className="text-xs text-destructive">{errors.tone}</p> : null}
    </div>
  );
}
