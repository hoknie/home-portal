"use client";

import type { ServiceView } from "@/entities/service";
import type { Section } from "@/shared/api";
import { cn } from "@/shared/lib/cn";
import { Heading, ListTransition, SURFACE } from "@/shared/ui/kit";
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
    <div className="grid gap-6">
      {placeWidgets(sections, widgets).map(({ section, widgets: placed }) => {
        const titled = Boolean(section.title) && section.appearance.title === "shown";
        return (
          <section
            key={section.id}
            className={cn("grid gap-4", section.appearance.surface === "card" && cn(SURFACE.panel, "rounded-2xl p-4 sm:p-5"))}
            aria-label={section.title ?? undefined}
            data-section={section.id}
            data-surface={section.appearance.surface}
          >
            {titled ? (
              <Heading level="group" as="h2" className="tracking-wide text-muted-foreground uppercase">
                {section.title}
              </Heading>
            ) : null}
            <div className={SECTION_GRID}>
              <ListTransition items={placed} keyOf={(widget) => widget.key}>
                {(widget) => (
                  <GridCell width={widget.width} height={widget.height} position={positionOf(widget)} memory={widget.key}>
                    <BoardWidget widget={widget} services={services} scope={scope} />
                  </GridCell>
                )}
              </ListTransition>
            </div>
          </section>
        );
      })}
    </div>
  );
}
