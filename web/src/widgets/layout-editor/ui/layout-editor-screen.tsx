"use client";

import { useLocale, useTranslations } from "next-intl";
import { useMemo } from "react";

import { useEnvironment } from "@/entities/environment";
import { groupsOf, useServices } from "@/entities/service";
import { LayoutEditor, type SettingsEditorProps, type WidgetKind } from "@/features/layout-editor";
import { BoardGrid, SETTINGS, SIZE_SPANS, SettingsForm, WIDGETS } from "@/features/widget-board";
import { PageHeader } from "@/shared/ui/page-header";

import { previewServices, previewWidgets } from "../model/preview";

export const INTERNET = "internet";

export function LayoutEditorScreen() {
  const t = useTranslations();
  const locale = useLocale();
  const environment = useEnvironment();
  const services = useServices();
  const groupsKey = groupsOf(services.data?.data.services ?? [], locale).join("\n");
  const groups = useMemo(() => (groupsKey === "" ? [] : groupsKey.split("\n")), [groupsKey]);
  const kinds: WidgetKind[] = useMemo(
    () =>
      Object.entries(WIDGETS).map(([type, entry]) => ({
        type,
        title: t(entry.titleKey),
        editor: SETTINGS[type]
          ? function Settings(props: SettingsEditorProps) {
              return <SettingsForm type={type} groups={groups} {...props} />;
            }
          : null,
      })),
    [t, groups],
  );
  const known = environment.data?.environments ?? [];
  const environments = known.includes(INTERNET) ? known : [...known, INTERNET];
  return (
    <div className="grid gap-8">
      <PageHeader title={t("layoutEditor.title")} description={t("layoutEditor.subtitle")} />
      <p className="-mt-4 text-sm text-muted-foreground">{t("layoutEditor.examples")}</p>
      <LayoutEditor
        kinds={kinds}
        environments={environments}
        spanOf={(size) => SIZE_SPANS[size]}
        renderPreview={(draft, scope) => {
          const widgets = previewWidgets(draft, scope);
          if (widgets.length === 0) {
            return <p className="text-sm text-muted-foreground">{t("layoutEditor.previewEmpty")}</p>;
          }
          return (
            <BoardGrid
              sections={draft.sections}
              widgets={widgets}
              services={previewServices(services.data?.data.services ?? [], scope)}
              scope={scope.signedIn ? "private" : "public"}
            />
          );
        }}
      />
    </div>
  );
}
