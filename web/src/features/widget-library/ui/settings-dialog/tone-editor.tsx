"use client";

import { useTranslations } from "next-intl";

import { Button, Input } from "@/shared/ui/primitives";

import { type RawBlock, type ToneMode, toneModeOf, withToneMode } from "../../model/blocks";
import { NATIVE_SELECT as SELECT } from "../../model/select-style";

const TONES = ["neutral", "ok", "info", "warning", "danger"] as const;

type Thresholds = { warning?: number; danger?: number; direction?: "above" | "below" };

export function ToneEditor({ id, block, onChange }: { id: string; block: RawBlock; onChange: (block: RawBlock) => void }) {
  const t = useTranslations("layoutEditor.blocks.tone");
  const mode = toneModeOf(block);
  const thresholds = (block.thresholds ?? {}) as Thresholds;
  const tones = Object.entries((block.tones ?? {}) as Record<string, string>);
  const number = (text: string) => (text === "" ? undefined : Number(text));
  return (
    <fieldset className="grid gap-2 rounded-lg border border-dashed p-3">
      <legend className="px-1 text-xs font-medium">{t("label")}</legend>
      <select aria-label={t("label")} className={SELECT} value={mode} onChange={(event) => onChange(withToneMode(block, event.target.value as ToneMode))}>
        {(["fixed", "thresholds", "tones"] as const).map((value) => (
          <option key={value} value={value}>
            {t(`modes.${value}`)}
          </option>
        ))}
      </select>
      {mode === "fixed" ? (
        <select id={`${id}-tone`} aria-label={t("modes.fixed")} className={SELECT} value={(block.tone as string | undefined) ?? "neutral"} onChange={(event) => onChange({ ...block, tone: event.target.value })}>
          {TONES.map((tone) => (
            <option key={tone} value={tone}>
              {t(`names.${tone}`)}
            </option>
          ))}
        </select>
      ) : null}
      {mode === "thresholds" ? (
        <div className="grid gap-2 @md:grid-cols-3">
          <label className="grid gap-1 text-xs">
            {t("warning")}
            <Input type="number" value={thresholds.warning ?? ""} onChange={(event) => onChange({ ...block, thresholds: { ...thresholds, warning: number(event.target.value) } })} />
          </label>
          <label className="grid gap-1 text-xs">
            {t("danger")}
            <Input type="number" value={thresholds.danger ?? ""} onChange={(event) => onChange({ ...block, thresholds: { ...thresholds, danger: number(event.target.value) } })} />
          </label>
          <label className="grid gap-1 text-xs">
            {t("direction")}
            <select className={SELECT} value={thresholds.direction ?? "above"} onChange={(event) => onChange({ ...block, thresholds: { ...thresholds, direction: event.target.value as "above" | "below" } })}>
              <option value="above">{t("directions.above")}</option>
              <option value="below">{t("directions.below")}</option>
            </select>
          </label>
        </div>
      ) : null}
      {mode === "tones" ? (
        <div className="grid gap-2">
          {tones.map(([value, tone], index) => (
            <div key={index} className="flex gap-2">
              <Input
                aria-label={t("map")}
                value={value}
                onChange={(event) => onChange({ ...block, tones: Object.fromEntries(tones.map(([key, chosen], position) => (position === index ? [event.target.value, chosen] : [key, chosen]))) })}
              />
              <select
                aria-label={t("label")}
                className={SELECT}
                value={tone}
                onChange={(event) => onChange({ ...block, tones: Object.fromEntries(tones.map(([key, chosen], position) => [key, position === index ? event.target.value : chosen])) })}
              >
                {TONES.map((name) => (
                  <option key={name} value={name}>
                    {t(`names.${name}`)}
                  </option>
                ))}
              </select>
            </div>
          ))}
          <Button type="button" variant="ghost" size="sm" className="w-fit" onClick={() => onChange({ ...block, tones: { ...Object.fromEntries(tones), "": "ok" } })}>
            {t("addTone")}
          </Button>
        </div>
      ) : null}
    </fieldset>
  );
}
