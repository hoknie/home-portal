import { GRID_COLUMNS, LARGEST_ROWS, type WidgetHeight } from "@/shared/api";

export type Axis = "width" | "height" | "both";

export type Size = { width: number; height: WidgetHeight };

export const ROW_PIXELS = 80;

export const GAP_PIXELS = 16;

export function clampColumns(columns: number) {
  return Math.min(GRID_COLUMNS, Math.max(1, Math.round(columns)));
}

export function snapColumns(start: number, deltaPixels: number, gridWidth: number): number {
  if (!(gridWidth > 0)) {
    return start;
  }
  return clampColumns(start + (deltaPixels / gridWidth) * GRID_COLUMNS);
}

export function stepColumns(columns: number, delta: number): number {
  return clampColumns(columns + delta);
}

export function rowsOfPixels(pixels: number) {
  return (pixels + GAP_PIXELS) / (ROW_PIXELS + GAP_PIXELS);
}

export function snapRows(startPixels: number, deltaPixels: number): WidgetHeight {
  const rows = rowsOfPixels(startPixels + deltaPixels);
  if (rows < 0.75) {
    return "auto";
  }
  return Math.min(LARGEST_ROWS, Math.max(1, Math.round(rows)));
}

export function stepRows(height: WidgetHeight, delta: number): WidgetHeight {
  const rows = height === "auto" ? 0 : height;
  const next = Math.min(LARGEST_ROWS, rows + delta);
  return next < 1 ? "auto" : next;
}

export function snapSize(start: Size, axis: Axis, delta: { x: number; y: number }, measure: { gridWidth: number; startPixels: number }): Size {
  return {
    width: axis === "height" ? start.width : snapColumns(start.width, delta.x, measure.gridWidth),
    height: axis === "width" ? start.height : snapRows(measure.startPixels, delta.y),
  };
}

const HORIZONTAL: Record<string, number> = { ArrowRight: 1, ArrowLeft: -1 };

const VERTICAL: Record<string, number> = { ArrowDown: 1, ArrowUp: -1 };

export function steppedSize(size: Size, axis: Axis, key: string): Size | null {
  if (axis === "width") {
    const step = HORIZONTAL[key] ?? { ArrowUp: 1, ArrowDown: -1 }[key];
    if (step !== undefined) {
      return { ...size, width: stepColumns(size.width, step) };
    }
    return key === "Home" ? { ...size, width: 1 } : key === "End" ? { ...size, width: GRID_COLUMNS } : null;
  }
  if (axis === "height") {
    const step = VERTICAL[key] ?? { ArrowRight: 1, ArrowLeft: -1 }[key];
    if (step !== undefined) {
      return { ...size, height: stepRows(size.height, step) };
    }
    return key === "Home" ? { ...size, height: "auto" } : key === "End" ? { ...size, height: LARGEST_ROWS } : null;
  }
  if (key in HORIZONTAL) {
    return { ...size, width: stepColumns(size.width, HORIZONTAL[key]) };
  }
  if (key in VERTICAL) {
    return { ...size, height: stepRows(size.height, VERTICAL[key]) };
  }
  return null;
}

export function sameSize(left: Size, right: Size) {
  return left.width === right.width && left.height === right.height;
}
