"use client";

import { useTranslations } from "next-intl";

import { ACCENTS, ALIGNS, type Appearance, PADDINGS, SURFACES, TITLE_VISIBILITIES } from "@/shared/api";
import { cn } from "@/shared/lib/cn";

import { ChoiceGroup } from "@/shared/ui/choice-group";

const SURFACE_SWATCH: Record<Appearance["surface"], string> = {
  card: "glass-panel",
  plain: "border border-dashed border-transparent",
  tinted: "widget-tint",
  outline: "border",
};

export function LookTab({ value, onChange }: { value: Appearance; onChange: (value: Appearance) => void }) {
  const t = useTranslations("layoutEditor.look");
  const set = <Key extends keyof Appearance>(key: Key) => (chosen: Appearance[Key]) => onChange({ ...value, [key]: chosen });
  return (
    <div className="grid gap-5" data-accent={value.accent}>
      <ChoiceGroup
        label={t("surface")}
        value={value.surface}
        onChange={set("surface")}
        choices={SURFACES.map((surface) => ({ value: surface, label: t(`surfaces.${surface}`), swatch: <span aria-hidden className={cn("h-6 w-10 rounded-md", SURFACE_SWATCH[surface])} /> }))}
      />
      <ChoiceGroup
        label={t("accent")}
        value={value.accent}
        onChange={set("accent")}
        choices={ACCENTS.map((accent) => ({ value: accent, label: t(`accents.${accent}`), swatch: <span aria-hidden data-accent={accent} className="size-5 rounded-full bg-widget-accent" /> }))}
      />
      <ChoiceGroup label={t("title")} value={value.title} onChange={set("title")} choices={TITLE_VISIBILITIES.map((title) => ({ value: title, label: t(`titles.${title}`) }))} />
      <ChoiceGroup label={t("padding")} value={value.padding} onChange={set("padding")} choices={PADDINGS.map((padding) => ({ value: padding, label: t(`paddings.${padding}`) }))} />
      <ChoiceGroup label={t("align")} value={value.align} onChange={set("align")} choices={ALIGNS.map((align) => ({ value: align, label: t(`aligns.${align}`) }))} />
    </div>
  );
}
