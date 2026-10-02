"use client";

import { useTranslations } from "next-intl";

import { ACCENTS, ALIGNS, type Appearance, PADDINGS, SURFACES, TITLE_VISIBILITIES } from "@/shared/api";
import { cn } from "@/shared/lib/cn";
import { IconChoice, SURFACE } from "@/shared/ui/kit";

import { ALIGN_ICONS, PADDING_ICONS, TITLE_ICONS } from "./choice-icons";

const SURFACE_SWATCH: Record<Appearance["surface"], string> = {
  card: SURFACE.panel,
  plain: "border border-dashed border-transparent",
  tinted: "widget-tint",
  outline: "border",
};

export function LookTab({ value, onChange }: { value: Appearance; onChange: (value: Appearance) => void }) {
  const t = useTranslations("layoutEditor.look");
  const set = <Key extends keyof Appearance>(key: Key) => (chosen: Appearance[Key]) => onChange({ ...value, [key]: chosen });
  return (
    <div className="grid gap-5" data-accent={value.accent}>
      <IconChoice
        label={t("surface")}
        value={value.surface}
        onChange={set("surface")}
        options={SURFACES.map((surface) => ({ value: surface, label: t(`surfaces.${surface}`), icon: <span aria-hidden className={cn("h-4 w-6 rounded-sm", SURFACE_SWATCH[surface])} /> }))}
      />
      <IconChoice
        label={t("accent")}
        value={value.accent}
        onChange={set("accent")}
        swatch
        options={ACCENTS.map((accent) => ({ value: accent, label: t(`accents.${accent}`), icon: <span aria-hidden data-accent={accent} className="size-full rounded-full bg-widget-accent" /> }))}
      />
      <IconChoice label={t("title")} value={value.title} onChange={set("title")} options={TITLE_VISIBILITIES.map((title) => ({ value: title, label: t(`titles.${title}`), icon: TITLE_ICONS[title] }))} />
      <IconChoice label={t("padding")} value={value.padding} onChange={set("padding")} options={PADDINGS.map((padding) => ({ value: padding, label: t(`paddings.${padding}`), icon: PADDING_ICONS[padding] }))} />
      <IconChoice label={t("align")} value={value.align} onChange={set("align")} options={ALIGNS.map((align) => ({ value: align, label: t(`aligns.${align}`), icon: ALIGN_ICONS[align] }))} />
    </div>
  );
}
