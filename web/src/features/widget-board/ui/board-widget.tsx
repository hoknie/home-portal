"use client";

import { useTranslations } from "next-intl";

import type { Widget } from "@/entities/dashboard";
import { type ModuleName, enabledModules, useModules } from "@/entities/module";
import type { ServiceView } from "@/entities/service";
import { WidgetFrame } from "@/shared/ui/widget-frame";

import { WIDGETS } from "../model/registry";
import { DataWidget } from "./data-widget";
import { ModuleOffWidget, UnknownWidget } from "./unknown-widget";

export type BoardWidgetProps = {
  widget: Pick<Widget, "type" | "id" | "title" | "settings"> & Partial<Pick<Widget, "appearance" | "height">>;
  services: ServiceView[];
  scope?: "private" | "public";
};

export function BoardWidget(props: BoardWidgetProps) {
  const needed = WIDGETS[props.widget.type]?.module;
  if (needed && (props.scope ?? "private") === "private") {
    return <ModuleChecked needed={needed} {...props} />;
  }
  return <DrawnWidget {...props} />;
}

function ModuleChecked({ needed, ...props }: BoardWidgetProps & { needed: ModuleName }) {
  const modules = useModules().data?.data;
  if (modules !== undefined && !enabledModules(modules).has(needed)) {
    return <ModuleOffWidget type={props.widget.type} />;
  }
  return <DrawnWidget {...props} />;
}

function DrawnWidget({ widget, services, scope = "private" }: BoardWidgetProps) {
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
  const fill = widget.height !== undefined && widget.height !== "auto";
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
        appearance={widget.appearance}
        fill={fill}
      />
    );
  }
  const Component = entry.component;
  return (
    <WidgetFrame title={title} appearance={widget.appearance} fill={fill}>
      <Component settings={settings.data} services={services} data={undefined} scope={scope} id={widget.id} />
    </WidgetFrame>
  );
}
