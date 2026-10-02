import type { CSSProperties } from "react";

import { GRID_COLUMNS, LARGEST_ROWS, type WidgetHeight } from "@/shared/api";

export const ROW_PIXELS = 80;

export const GAP_PIXELS = 16;

export const HALF_ROW = 6;

export const STEP_PIXELS = 8;

export const STEPS_PER_ROW = (ROW_PIXELS + GAP_PIXELS) / STEP_PIXELS;

export const SECTION_GRID = "-mb-4 grid grid-cols-12 auto-rows-[8px] gap-x-4";

export const CELL_CLASSES = "col-span-12 min-w-0 pb-4 sm:[grid-column:span_var(--tablet-span)] lg:[grid-column:span_var(--span)] [grid-row:span_var(--rows)]";

export const PLACED_CLASSES =
  "col-span-12 min-w-0 pb-4 sm:[grid-column:span_var(--tablet-span)] lg:[grid-column:var(--column)/span_var(--span)] [grid-row:span_var(--rows)]";

export type Position = { column: number; row: number } | null;

export function positionOf(widget: { column?: number | null; row?: number | null }): Position {
  return typeof widget.column === "number" && typeof widget.row === "number" ? { column: widget.column, row: widget.row } : null;
}

export function cellClasses(position: Position) {
  return position === null ? CELL_CLASSES : PLACED_CLASSES;
}

export function inReadingOrder<Widget extends { column?: number | null; row?: number | null }>(widgets: readonly Widget[]): Widget[] {
  const placed = widgets.filter((widget) => positionOf(widget) !== null);
  const flowing = widgets.filter((widget) => positionOf(widget) === null);
  const order = (widget: Widget) => {
    const position = positionOf(widget);
    return position === null ? 0 : position.row * 100 + position.column;
  };
  return [...[...placed].sort((left, right) => order(left) - order(right)), ...flowing];
}

export function columnsOf(width: number) {
  return Math.min(GRID_COLUMNS, Math.max(1, Math.round(width)));
}

export function tabletSpanOf(width: number) {
  return columnsOf(width) <= HALF_ROW ? HALF_ROW : GRID_COLUMNS;
}

export function rowsFor(contentPixels: number) {
  return Math.min(LARGEST_ROWS * 4 * STEPS_PER_ROW, Math.max(1, Math.ceil((contentPixels + GAP_PIXELS) / STEP_PIXELS)));
}

export function cellStyle(width: number, height: WidgetHeight, measuredRows: number, position: Position = null): CSSProperties {
  const rows = height === "auto" ? measuredRows : height * STEPS_PER_ROW;
  const placed = position === null ? {} : { "--column": position.column, "--row": position.row };
  return { "--span": columnsOf(width), "--tablet-span": tabletSpanOf(width), "--rows": rows, ...placed } as unknown as CSSProperties;
}
