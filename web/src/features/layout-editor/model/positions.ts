import { GRID_COLUMNS } from "@/shared/api";

import { type Draft, widgetsOf } from "./draft";

export type Cell = { column: number; row: number; width: number; rows: number };

export function overlaps(left: Cell, right: Cell) {
  return left.column < right.column + right.width && right.column < left.column + left.width && left.row < right.row + right.rows && right.row < left.row + left.rows;
}

export function clampedCell(cell: Cell): Cell {
  const width = Math.min(GRID_COLUMNS, Math.max(1, cell.width));
  return { ...cell, width, column: Math.min(GRID_COLUMNS - width + 1, Math.max(1, cell.column)), row: Math.max(1, cell.row) };
}

export function settled(cells: Record<string, Cell>, moved: string): Record<string, Cell> {
  const fixed: [string, Cell][] = [[moved, cells[moved]]];
  const others = Object.entries(cells)
    .filter(([uid]) => uid !== moved)
    .sort(([, left], [, right]) => left.row - right.row || left.column - right.column);
  for (const [uid, start] of others) {
    let cell = start;
    let blocking = fixed.find(([, other]) => overlaps(cell, other));
    while (blocking) {
      const below = blocking[1];
      cell = { ...cell, row: below.row + below.rows };
      blocking = fixed.find(([, other]) => overlaps(cell, other));
    }
    fixed.push([uid, cell]);
  }
  return Object.fromEntries(fixed);
}

export function movedTo(draft: Draft, uid: string, section: string, target: { column: number; row: number }, measured: Record<string, Cell>): Draft {
  const moving = draft.widgets.find((widget) => widget.uid === uid);
  if (!moving || !draft.sections.some((candidate) => candidate.id === section)) {
    return draft;
  }
  const rows = measured[uid]?.rows ?? (moving.height === "auto" ? 1 : moving.height);
  const cell = clampedCell({ column: target.column, row: target.row, width: moving.width, rows });
  const neighbours = Object.fromEntries(
    widgetsOf(draft, section)
      .filter((widget) => widget.uid !== uid && measured[widget.uid] !== undefined)
      .map((widget) => [widget.uid, measured[widget.uid]]),
  );
  const placed = settled({ ...neighbours, [uid]: cell }, uid);
  const widgets = draft.widgets.map((widget) => {
    const position = placed[widget.uid];
    if (!position) {
      return widget;
    }
    return { ...widget, section: widget.uid === uid ? section : widget.section, column: position.column, row: position.row };
  });
  return { ...draft, widgets: draft.sections.flatMap((candidate) => widgets.filter((widget) => widget.section === candidate.id)) };
}
