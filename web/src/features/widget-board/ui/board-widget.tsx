"use client";

import { useTranslations } from "next-intl";

import type { Widget } from "@/entities/dashboard";
import type { ServiceView } from "@/entities/service";
import { WidgetFrame } from "@/shared/ui/widget-frame";

import { WIDGETS } from "../model/registry";
import { DataWidget } from "./data-widget";
import { UnknownWidget } from "./unknown-widget";

export type BoardWidgetProps = {
  widget: Pick<Widget, "type" | "id" | "title" | "settings">;
  services: ServiceView[];
  scope?: "private" | "public";
};

export function BoardWidget({ widget, services, scope = "private" }: BoardWidgetProps) {
  const t = useTranslations();
  const entry = WIDGETS[widget.type];
  if (!entry) {
    return <UnknownWidget type={widget.type} invalid={false} />;
  }
  const settings = entry.schema.safeParse(widget.settings);
  if (!settings.success) {
    return <UnknownWidget type={widget.type} invalid />;
  }
  const title = widget.title ?? t(entry.titleKey);
  if (entry.data) {
    if (!widget.id) {
      return <UnknownWidget type={widget.type} invalid />;
    }
    return (
      <DataWidget
        entry={entry}
        id={widget.id}
        title={title}
        settings={settings.data}
        services={services}
        scope={scope}
      />
    );
  }
  const Component = entry.component;
  return (
    <WidgetFrame title={title}>
      <Component settings={settings.data} services={services} data={undefined} scope={scope} />
    </WidgetFrame>
  );
}
