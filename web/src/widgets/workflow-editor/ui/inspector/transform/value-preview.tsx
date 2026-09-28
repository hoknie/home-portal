"use client";

import { useTranslations } from "next-intl";

import { type Preview, canonical, typeOfValue } from "@/entities/workflow";

export const LONGEST_PREVIEW = 400;

export function previewText(value: unknown) {
  const text = typeof value === "string" ? JSON.stringify(value) : canonical(value);
  return text.length > LONGEST_PREVIEW ? `${text.slice(0, LONGEST_PREVIEW - 1)}…` : text;
}

export type ValuePreviewProps = { preview: Preview | null; from: "sample" | "run" | null; label: string };

export function ValuePreview({ preview, from, label }: ValuePreviewProps) {
  const t = useTranslations("workflowEditor.transform");
  if (preview === null) {
    return <p className="text-xs text-muted-foreground">{t("noSample")}</p>;
  }
  if (preview.error !== null) {
    return (
      <p role="alert" className="rounded-md border border-destructive/40 bg-destructive/10 px-2 py-1 text-xs text-destructive">
        {t("cannot", { error: preview.error })}
      </p>
    );
  }
  return (
    <figure className="grid gap-1" aria-label={label}>
      <figcaption className="flex items-center gap-2 text-xs text-muted-foreground">
        <span>{label}</span>
        <span className="rounded-sm bg-primary/12 px-1.5 font-mono text-[11px] text-primary">{t(`types.${typeOfValue(preview.value)}`)}</span>
        {from ? <span>{t(from === "sample" ? "fromSample" : "fromRun")}</span> : null}
      </figcaption>
      <pre className="max-h-32 overflow-auto rounded-md bg-glass-tint px-2 py-1 font-mono text-xs break-all whitespace-pre-wrap">{previewText(preview.value)}</pre>
    </figure>
  );
}
