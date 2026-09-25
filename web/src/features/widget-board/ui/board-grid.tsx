"use client";

import type { ServiceView } from "@/entities/service";
import type { Section } from "@/shared/api";
import { cn } from "@/shared/lib/cn";

import { type GridWidget, SIZE_SPANS, placeWidgets } from "../model/layout";
import { BoardWidget } from "./board-widget";

export type BoardGridProps = {
  sections: Section[];
  widgets: GridWidget[];
  services: ServiceView[];
  scope?: "private" | "public";
};

export function BoardGrid({ sections, widgets, services, scope = "private" }: BoardGridProps) {
  return (
    <div className="grid gap-10">
      {placeWidgets(sections, widgets).map(({ section, widgets: placed }) => (
        <section key={section.id} className="grid gap-4" aria-label={section.title ?? undefined} data-section={section.id}>
          {section.title ? <h2 className="text-sm font-semibold tracking-wide text-muted-foreground uppercase">{section.title}</h2> : null}
          <div className="grid grid-cols-12 gap-4">
            {placed.map((widget) => (
              <div key={widget.key} className={cn("@container min-w-0", SIZE_SPANS[widget.size])} data-size={widget.size}>
                <BoardWidget widget={widget} services={services} scope={scope} />
              </div>
            ))}
          </div>
        </section>
      ))}
    </div>
  );
}
