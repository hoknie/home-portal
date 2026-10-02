import type { z } from "zod";

import { type Appearance, type WidgetHeight, generated } from "@/shared/api";

export const librarySchema = generated.dashboardLibrary.libraryResponseSchema;

export const libraryWidgetSchema = generated.dashboardLibrary.libraryWidgetViewSchema;

export type LibraryWidget = z.infer<typeof libraryWidgetSchema>;

export type LibraryEntry = {
  id: string | null;
  type: string;
  title: string | null;
  settings: Record<string, unknown>;
  environments: string[] | null;
  public: boolean;
  appearance: Appearance;
  width?: number;
  height?: WidgetHeight;
};

export function entryOf(widget: LibraryWidget): LibraryEntry {
  return { id: widget.id, type: widget.type, title: widget.title, settings: widget.settings, environments: widget.environments, public: widget.public, appearance: widget.appearance, width: widget.width, height: widget.height };
}
