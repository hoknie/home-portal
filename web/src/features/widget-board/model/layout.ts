import type { Appearance, Section, WidgetHeight } from "@/shared/api";
import { inReadingOrder } from "@/shared/lib/widget-grid";

export type GridWidget = {
  key: string;
  type: string;
  id: string | null;
  title: string | null;
  settings: Record<string, unknown>;
  section: string | null;
  width: number;
  height: WidgetHeight;
  column: number | null;
  row: number | null;
  appearance: Appearance;
};

export type PlacedSection = { section: Section; widgets: GridWidget[] };

export function placeWidgets(sections: Section[], widgets: GridWidget[]): PlacedSection[] {
  const first = sections[0]?.id ?? null;
  const known = new Set(sections.map((section) => section.id));
  return sections
    .map((section) => ({
      section,
      widgets: inReadingOrder(
        widgets.filter((widget) => {
          const placed = widget.section && known.has(widget.section) ? widget.section : first;
          return placed === section.id;
        }),
      ),
    }))
    .filter((placed) => placed.widgets.length > 0);
}
