import { GRID_COLUMNS } from "@/shared/api";
import { inReadingOrder } from "@/shared/lib/widget-grid";

import { type Draft, type DraftWidget, widgetsOf } from "./draft";

export type Place = { section: string; column: number; before: string | null };

export type Step = { columns: number; rows: number; sections: number };

export type Standing = { section: string; column: number; index: number; count: number };

export function ordered(draft: Draft, section: string): DraftWidget[] {
  return inReadingOrder(widgetsOf(draft, section));
}

export function fittedColumn(column: number, width: number): number {
  return Math.min(GRID_COLUMNS - Math.min(GRID_COLUMNS, width) + 1, Math.max(1, Math.round(column)));
}

function numbered(widgets: DraftWidget[], columns: Record<string, number>): DraftWidget[] {
  return widgets.map((widget, index) => ({ ...widget, column: widget.column ?? columns[widget.uid] ?? 1, row: index + 1 }));
}

export function placedAt(draft: Draft, uid: string, place: Place, columns: Record<string, number> = {}): Draft {
  const moving = draft.widgets.find((widget) => widget.uid === uid);
  if (!moving || !draft.sections.some((section) => section.id === place.section) || place.before === uid) {
    return draft;
  }
  const lists = new Map(draft.sections.map((section) => [section.id, ordered(draft, section.id).filter((widget) => widget.uid !== uid)]));
  const target = lists.get(place.section) ?? [];
  const at = place.before === null ? target.length : target.findIndex((widget) => widget.uid === place.before);
  target.splice(at < 0 ? target.length : at, 0, { ...moving, section: place.section, column: fittedColumn(place.column, moving.width) });
  const touched = new Set([moving.section, place.section]);
  const renumbered = new Map(draft.sections.map((section) => [section.id, touched.has(section.id) ? numbered(lists.get(section.id) ?? [], columns) : (lists.get(section.id) ?? [])]));
  return { ...draft, widgets: draft.sections.flatMap((section) => renumbered.get(section.id) ?? []) };
}

export function standingOf(draft: Draft, uid: string, columns: Record<string, number> = {}): Standing | null {
  const widget = draft.widgets.find((candidate) => candidate.uid === uid);
  if (!widget) {
    return null;
  }
  const list = ordered(draft, widget.section);
  return { section: widget.section, column: widget.column ?? columns[uid] ?? 1, index: list.findIndex((candidate) => candidate.uid === uid), count: list.length };
}

export function steppedPlace(draft: Draft, uid: string, step: Step, columns: Record<string, number> = {}): Place | null {
  const standing = standingOf(draft, uid, columns);
  if (!standing) {
    return null;
  }
  if (step.sections !== 0) {
    const index = draft.sections.findIndex((section) => section.id === standing.section);
    const next = draft.sections[Math.min(draft.sections.length - 1, Math.max(0, index + step.sections))];
    return next && next.id !== standing.section ? { section: next.id, column: standing.column, before: null } : null;
  }
  const others = ordered(draft, standing.section).filter((widget) => widget.uid !== uid);
  const index = Math.min(others.length, Math.max(0, standing.index + step.rows));
  return { section: standing.section, column: standing.column + step.columns, before: others[index]?.uid ?? null };
}
