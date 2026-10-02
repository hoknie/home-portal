import { GAP_PIXELS, ROW_PIXELS } from "@/shared/lib/widget-grid";

import type { Cell } from "./positions";

export const SECTION_GRID_ATTRIBUTE = "data-section-grid";

const COLUMNS = 12;

export type Target = { section: string; column: number; row: number };

function gridOf(section: string) {
  return document.querySelector<HTMLElement>(`[${SECTION_GRID_ATTRIBUTE}="${CSS.escape(section)}"]`);
}

function step(grid: HTMLElement) {
  const rect = grid.getBoundingClientRect();
  return { rect, column: (rect.width - (COLUMNS - 1) * GAP_PIXELS) / COLUMNS + GAP_PIXELS, row: ROW_PIXELS + GAP_PIXELS };
}

export function cellsIn(section: string): Record<string, Cell> {
  const grid = gridOf(section);
  if (!grid) {
    return {};
  }
  const { rect, column, row } = step(grid);
  if (!(column > GAP_PIXELS)) {
    return {};
  }
  const cells: Record<string, Cell> = {};
  for (const tile of grid.querySelectorAll<HTMLElement>(":scope > [data-widget]")) {
    const uid = tile.getAttribute("data-widget");
    const box = tile.getBoundingClientRect();
    if (uid === null) {
      continue;
    }
    cells[uid] = {
      column: Math.round((box.left - rect.left) / column) + 1,
      row: Math.round((box.top - rect.top) / row) + 1,
      width: Math.max(1, Math.round((box.width + GAP_PIXELS) / column)),
      rows: Math.max(1, Math.round((box.height + GAP_PIXELS) / row)),
    };
  }
  return cells;
}

export function sectionAt(x: number, y: number): string | null {
  const found = typeof document.elementFromPoint === "function" ? document.elementFromPoint(x, y) : null;
  return found?.closest(`[${SECTION_GRID_ATTRIBUTE}]`)?.getAttribute(SECTION_GRID_ATTRIBUTE) ?? null;
}

export function cellAt(section: string, x: number, y: number, grab: { columns: number; rows: number }): { column: number; row: number } | null {
  const grid = gridOf(section);
  if (!grid) {
    return null;
  }
  const { rect, column, row } = step(grid);
  if (!(column > GAP_PIXELS)) {
    return null;
  }
  return {
    column: Math.floor((x - rect.left) / column) + 1 - grab.columns,
    row: Math.max(1, Math.floor((y - rect.top) / row) + 1 - grab.rows),
  };
}

export function grabOf(tile: HTMLElement, section: string, x: number, y: number) {
  const grid = gridOf(section);
  if (!grid) {
    return { columns: 0, rows: 0 };
  }
  const { column, row } = step(grid);
  const box = tile.getBoundingClientRect();
  return { columns: Math.max(0, Math.floor((x - box.left) / column)), rows: Math.max(0, Math.floor((y - box.top) / row)) };
}
