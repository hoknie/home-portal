"use client";

import { useTranslations } from "next-intl";

import type { ServiceView } from "@/entities/service";
import { isPending, useWidgetData } from "@/entities/widget";
import type { Appearance } from "@/shared/api";
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
  appearance?: Appearance;
  fill?: boolean;
};

export function DataWidget({ entry, id, title, settings, services, scope, appearance, fill }: DataWidgetProps) {
  const framed = { title, appearance, fill };
  const t = useTranslations("widgets");
  const query = useWidgetData(id, scope);
  const answer = isPending(query.data) ? null : query.data;
  const parsed = answer && entry.data ? entry.data.safeParse(answer.data) : null;
  if (query.isPending || isPending(query.data)) {
    return <WidgetFrame {...framed} loading>{null}</WidgetFrame>;
  }
  if (!answer || !parsed?.success) {
    return (
      <WidgetFrame {...framed}>
        <WidgetProblem message={answer?.problem ?? query.error?.message ?? t("failed")} />
      </WidgetFrame>
    );
  }
  const Component = entry.component;
  return (
    <WidgetFrame {...framed} stale={answer.stale} problem={answer.problem}>
      <div className="grid gap-2">
        <Component settings={settings} services={services} data={parsed.data} scope={scope} id={id} />
        <p className="flex justify-end gap-1 text-xs text-muted-foreground">
          <span>{t("updated")}</span>
          <RelativeTime moment={answer.fetched_at} />
        </p>
      </div>
    </WidgetFrame>
  );
}
