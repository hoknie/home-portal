"use client";

import { useTranslations } from "next-intl";

import { Button, IconChoice, Input, NativeSelect, TONE_NAMES } from "@/shared/ui/kit";

import { type RawBlock, type ToneMode, toneModeOf, withToneMode } from "../../model/blocks";
import { DIRECTION_ICONS, toneSwatch } from "./choice-icons";

type Thresholds = { warning?: number; danger?: number; direction?: "above" | "below" };

export function ToneEditor({ block, onChange }: { block: RawBlock; onChange: (block: RawBlock) => void }) {
  const t = useTranslations("layoutEditor.blocks.tone");
  const mode = toneModeOf(block);
  const thresholds = (block.thresholds ?? {}) as Thresholds;
  const tones = Object.entries((block.tones ?? {}) as Record<string, string>);
  const number = (text: string) => (text === "" ? undefined : Number(text));
  return (
    <fieldset className="grid gap-2 rounded-lg border border-dashed p-3">
      <legend className="px-1 text-xs font-medium">{t("label")}</legend>
      <NativeSelect aria-label={t("label")} value={mode} onChange={(event) => onChange(withToneMode(block, event.target.value as ToneMode))}>
        {(["fixed", "thresholds", "tones"] as const).map((value) => (
          <option key={value} value={value}>
            {t(`modes.${value}`)}
          </option>
        ))}
      </NativeSelect>
      {mode === "fixed" ? (
        <IconChoice
          label={t("modes.fixed")}
          value={(block.tone as string | undefined) ?? "neutral"}
          swatch
          onChange={(tone) => onChange({ ...block, tone })}
          options={TONE_NAMES.map((tone) => ({ value: tone, label: t(`names.${tone}`), icon: toneSwatch(tone) }))}
        />
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
          <IconChoice
            label={t("direction")}
            value={thresholds.direction ?? "above"}
            onChange={(direction) => onChange({ ...block, thresholds: { ...thresholds, direction } })}
            options={(["above", "below"] as const).map((direction) => ({ value: direction, label: t(`directions.${direction}`), icon: DIRECTION_ICONS[direction] }))}
          />
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
              <NativeSelect
                aria-label={t("label")}
                value={tone}
                onChange={(event) => onChange({ ...block, tones: Object.fromEntries(tones.map(([key, chosen], position) => [key, position === index ? event.target.value : chosen])) })}
              >
                {TONE_NAMES.map((name) => (
                  <option key={name} value={name}>
                    {t(`names.${name}`)}
                  </option>
                ))}
              </NativeSelect>
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
