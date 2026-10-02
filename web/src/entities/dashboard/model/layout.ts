import type { Section, WidgetHeight } from "@/shared/api";

export type LayoutWidgetRequest = {
  key: string | null;
  widget: string;
  section: string;
  column: number | null;
  row: number | null;
  width: number;
  height: WidgetHeight;
};

export type LayoutRequest = { sections: Section[]; widgets: LayoutWidgetRequest[] };
