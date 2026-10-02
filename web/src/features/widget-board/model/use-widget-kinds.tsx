"use client";

import type { LucideIcon } from "lucide-react";
import { useLocale, useTranslations } from "next-intl";
import { type ComponentType, useMemo } from "react";

import type { ModuleName } from "@/entities/module";
import { groupsOf, useServices } from "@/entities/service";

import { SettingsForm } from "../ui/settings-form";
import { WIDGETS } from "./registry";
import { SETTINGS, settingsErrors } from "./settings-fields";

export type KindEditorProps = { value: Record<string, unknown>; onChange: (value: Record<string, unknown>) => void };

export type KindInfo = {
  type: string;
  title: string;
  description: string;
  icon: LucideIcon;
  module: ModuleName | null;
  editor: ComponentType<KindEditorProps> | null;
  errors: (settings: Record<string, unknown>) => Record<string, string>;
};

export function useWidgetKinds(): KindInfo[] {
  const t = useTranslations();
  const locale = useLocale();
  const services = useServices();
  const groupsKey = groupsOf(services.data?.data.services ?? [], locale).join("\n");
  return useMemo(() => {
    const groups = groupsKey === "" ? [] : groupsKey.split("\n");
    return Object.entries(WIDGETS).map(([type, entry]) => ({
      type,
      title: t(entry.titleKey),
      description: t(entry.descriptionKey),
      icon: entry.icon,
      module: entry.module ?? null,
      errors: (settings: Record<string, unknown>) => settingsErrors(type, settings),
      editor: SETTINGS[type]
        ? function Settings(props: KindEditorProps) {
            return <SettingsForm type={type} groups={groups} {...props} />;
          }
        : null,
    }));
  }, [t, groupsKey]);
}
