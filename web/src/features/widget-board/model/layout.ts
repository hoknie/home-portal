import type { Section, WidgetSize } from "@/shared/api";

export type GridWidget = {
  key: string;
  type: string;
  id: string | null;
  title: string | null;
  settings: Record<string, unknown>;
  section: string | null;
  size: WidgetSize;
};

export type PlacedSection = { section: Section; widgets: GridWidget[] };

export const SIZE_SPANS: Record<WidgetSize, string> = {
  quarter: "col-span-12 sm:col-span-6 lg:col-span-3",
  third: "col-span-12 sm:col-span-6 lg:col-span-4",
  half: "col-span-12 sm:col-span-6",
  "two-thirds": "col-span-12 lg:col-span-8",
  full: "col-span-12",
};

export function placeWidgets(sections: Section[], widgets: GridWidget[]): PlacedSection[] {
  const first = sections[0]?.id ?? null;
  const known = new Set(sections.map((section) => section.id));
  return sections
    .map((section) => ({
      section,
      widgets: widgets.filter((widget) => {
        const placed = widget.section && known.has(widget.section) ? widget.section : first;
        return placed === section.id;
      }),
    }))
    .filter((placed) => placed.widgets.length > 0);
}
