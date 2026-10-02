"use client";

import { useTranslations } from "next-intl";

import { type ServiceView, useServices } from "@/entities/service";
import type { CustomWidgetData } from "@/entities/widget";
import type { Appearance, WidgetHeight } from "@/shared/api";
import { WidgetFrame } from "@/shared/ui/widget-frame";

import { BoardWidget } from "./board-widget";
import { CustomWidget, type Marking } from "./kinds/custom/custom-widget";

export type PreviewedWidget = { type: string; id: string | null; title: string | null; settings: Record<string, unknown>; appearance: Appearance };

export function LibraryPreview({ widget, custom, height = "auto", marking }: { widget: PreviewedWidget; custom?: CustomWidgetData | null; height?: WidgetHeight; marking?: Marking }) {
  const t = useTranslations();
  const services = (useServices().data?.data.services ?? []) as ServiceView[];
  if (custom) {
    return (
      <WidgetFrame title={widget.title ?? t("widgets.custom.title")} appearance={widget.appearance} fill={height !== "auto"}>
        <CustomWidget data={custom} scope="private" widget={widget.id ?? ""} marking={marking} />
      </WidgetFrame>
    );
  }
  return <BoardWidget widget={{ ...widget, height }} services={services} />;
}
