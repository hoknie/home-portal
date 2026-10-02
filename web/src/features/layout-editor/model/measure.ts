import { GAP_PIXELS } from "@/shared/lib/widget-grid";

import type { Place } from "./order";

export const SECTION_GRID_ATTRIBUTE = "data-section-grid";

const COLUMNS = 12;

export type Tile = { uid: string; top: number; height: number };

export type Snapshot = Record<string, Tile[]>;

function gridOf(section: string) {
  return document.querySelector<HTMLElement>(`[${SECTION_GRID_ATTRIBUTE}="${CSS.escape(section)}"]`);
}

function columnStep(grid: HTMLElement) {
  const rect = grid.getBoundingClientRect();
  return { rect, column: (rect.width - (COLUMNS - 1) * GAP_PIXELS) / COLUMNS + GAP_PIXELS };
}

function tilesOf(grid: HTMLElement) {
  return [...grid.querySelectorAll<HTMLElement>(":scope > [data-widget]")];
}

export function columnsIn(section: string): Record<string, number> {
  const grid = gridOf(section);
  if (!grid) {
    return {};
  }
  const { rect, column } = columnStep(grid);
  if (!(column > GAP_PIXELS)) {
    return {};
  }
  return Object.fromEntries(
    tilesOf(grid).flatMap((tile) => {
      const uid = tile.getAttribute("data-widget");
      return uid === null ? [] : [[uid, Math.round((tile.getBoundingClientRect().left - rect.left) / column) + 1]];
    }),
  );
}

export function snapshotOf(sections: string[]): Snapshot {
  return Object.fromEntries(
    sections.map((section) => {
      const grid = gridOf(section);
      const tiles = grid
        ? tilesOf(grid).flatMap((tile) => {
            const uid = tile.getAttribute("data-widget");
            const box = tile.getBoundingClientRect();
            return uid === null ? [] : [{ uid, top: box.top, height: box.height }];
          })
        : [];
      return [section, tiles];
    }),
  );
}

export function sectionAt(x: number, y: number): string | null {
  const found = typeof document.elementFromPoint === "function" ? document.elementFromPoint(x, y) : null;
  return found?.closest(`[${SECTION_GRID_ATTRIBUTE}]`)?.getAttribute(SECTION_GRID_ATTRIBUTE) ?? null;
}

export function grabbedColumns(tile: HTMLElement, section: string, x: number): number {
  const grid = gridOf(section);
  if (!grid) {
    return 0;
  }
  const { column } = columnStep(grid);
  return column > GAP_PIXELS ? Math.max(0, Math.floor((x - tile.getBoundingClientRect().left) / column)) : 0;
}

export function placeAt(snapshot: Snapshot, section: string, uid: string, x: number, y: number, grabbed: number): Place | null {
  const grid = gridOf(section);
  if (!grid) {
    return null;
  }
  const { rect, column } = columnStep(grid);
  if (!(column > GAP_PIXELS)) {
    return null;
  }
  const before = (snapshot[section] ?? []).find((tile) => tile.uid !== uid && tile.top + tile.height / 2 > y)?.uid ?? null;
  return { section, column: Math.floor((x - rect.left) / column) + 1 - grabbed, before };
}
