"use client";

import { useTranslations } from "next-intl";

import type { ServiceView } from "@/entities/service";
import { useWidgetData } from "@/entities/widget";
import { RelativeTime } from "@/shared/ui/relative-time";
import { WidgetFrame, WidgetProblem } from "@/shared/ui/widget-frame";

import type { WidgetEntry } from "../model/registry";

export type DataWidgetProps = {
  entry: WidgetEntry;
  id: string;
  title: string;
  settings: unknown;
  services: ServiceView[];
  scope: "private" | "public";
};

export function DataWidget({ entry, id, title, settings, services, scope }: DataWidgetProps) {
  const t = useTranslations("widgets");
  const query = useWidgetData(id, scope);
  const parsed = query.data && entry.data ? entry.data.safeParse(query.data.data) : null;
  if (query.isPending) {
    return <WidgetFrame title={title} loading>{null}</WidgetFrame>;
  }
  if (!query.data || !parsed?.success) {
    return (
      <WidgetFrame title={title}>
        <WidgetProblem message={query.data?.problem ?? query.error?.message ?? t("failed")} />
      </WidgetFrame>
    );
  }
  const Component = entry.component;
  return (
    <WidgetFrame title={title} stale={query.data.stale} problem={query.data.problem}>
      <div className="grid gap-2">
        <Component settings={settings} services={services} data={parsed.data} scope={scope} />
        <p className="flex justify-end gap-1 text-xs text-muted-foreground">
          <span>{t("updated")}</span>
          <RelativeTime moment={query.data.fetched_at} />
        </p>
      </div>
    </WidgetFrame>
  );
}
