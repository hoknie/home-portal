"use client";

import { useTranslations } from "next-intl";

import { useDashboard } from "@/entities/dashboard";
import type { Service } from "@/entities/service";
import { BoardGrid } from "@/features/widget-board";

export type RelatedWidgetsProps = { service: Service; services: Service[] };

export function RelatedWidgets({ service, services }: RelatedWidgetsProps) {
  const t = useTranslations("servicePage");
  const dashboard = useDashboard();
  const related = (dashboard.data?.widgets ?? []).filter((widget) => widget.id !== null && service.widgets.includes(widget.id));
  if (related.length === 0) {
    return null;
  }
  const section = { id: "related", title: t("related") };
  return (
    <BoardGrid
      sections={[section]}
      widgets={related.map((widget) => ({ ...widget, section: section.id }))}
      services={services}
    />
  );
}
