import type { Section, WidgetSize } from "@/shared/api";

export type LayoutWidgetRequest = {
  key: string | null;
  type: string;
  id: string | null;
  title: string | null;
  settings: Record<string, unknown>;
  environments: string[] | null;
  public: boolean;
  section: string;
  size: WidgetSize;
};

export type LayoutRequest = { sections: Section[]; widgets: LayoutWidgetRequest[] };
