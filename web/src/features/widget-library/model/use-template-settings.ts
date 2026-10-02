"use client";

import { useTranslations } from "next-intl";

import { CUSTOM_TEMPLATES, type TemplateText } from "./templates";

export function useTemplateSettings(key: string | null): Record<string, unknown> {
  const t = useTranslations("layoutEditor.gallery");
  const template = CUSTOM_TEMPLATES.find((candidate) => candidate.key === key);
  return template ? { blocks: template.blocks((text: TemplateText) => t(`templateTexts.${text}`)) } : { blocks: [] };
}
