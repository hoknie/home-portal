import { WIDGET_SIZES, type WidgetSize } from "@/shared/api";

export const GRID_COLUMNS = 12;

export const COLUMNS: Record<WidgetSize, number> = { quarter: 3, third: 4, half: 6, "two-thirds": 8, full: 12 };

export function snapSize(start: WidgetSize, deltaPixels: number, gridWidth: number): WidgetSize {
  if (!(gridWidth > 0)) {
    return start;
  }
  const wanted = COLUMNS[start] + (deltaPixels / gridWidth) * GRID_COLUMNS;
  return WIDGET_SIZES.reduce((best, size) => (Math.abs(COLUMNS[size] - wanted) < Math.abs(COLUMNS[best] - wanted) ? size : best), start);
}

export function stepSize(size: WidgetSize, delta: number): WidgetSize {
  const index = WIDGET_SIZES.indexOf(size);
  return WIDGET_SIZES[Math.max(0, Math.min(WIDGET_SIZES.length - 1, index + delta))];
}
