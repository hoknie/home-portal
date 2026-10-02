"use client";

import type { ServiceView } from "@/entities/service";
import type { Section } from "@/shared/api";
import { cn } from "@/shared/lib/cn";
import { SECTION_GRID, positionOf } from "@/shared/lib/widget-grid";

import { type GridWidget, placeWidgets } from "../model/layout";
import { BoardWidget } from "./board-widget";
import { GridCell } from "./grid-cell";

export type BoardGridProps = {
  sections: Section[];
  widgets: GridWidget[];
  services: ServiceView[];
  scope?: "private" | "public";
};

export function BoardGrid({ sections, widgets, services, scope = "private" }: BoardGridProps) {
  return (
    <div className="grid gap-10">
      {placeWidgets(sections, widgets).map(({ section, widgets: placed }) => {
        const titled = Boolean(section.title) && section.appearance.title === "shown";
        return (
          <section
            key={section.id}
            className={cn("grid gap-4", section.appearance.surface === "card" && "glass-panel rounded-2xl border p-4 sm:p-5")}
            aria-label={section.title ?? undefined}
            data-section={section.id}
            data-surface={section.appearance.surface}
          >
            {titled ? <h2 className="text-sm font-semibold tracking-wide text-muted-foreground uppercase">{section.title}</h2> : null}
            <div className={SECTION_GRID}>
              {placed.map((widget) => (
                <GridCell key={widget.key} width={widget.width} height={widget.height} position={positionOf(widget)}>
                  <BoardWidget widget={widget} services={services} scope={scope} />
                </GridCell>
              ))}
            </div>
          </section>
        );
      })}
    </div>
  );
}
