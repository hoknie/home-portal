import type { CollisionDetection, KeyboardCoordinateGetter } from "@dnd-kit/core";

import { type Draft, moveSection, moveWidget, widgetsOf } from "./draft";

export const SECTION_PREFIX = "section:";

export const SECTION_SORT_PREFIX = "sort-section:";

export const WIDGET_KIND = "widget";

export const SECTION_KIND = "section";

export const AREA_KIND = "area";

export type DragItem = { id: string; kind: string | null; section: string | null };

export function sectionOfSortable(id: string) {
  return id.startsWith(SECTION_SORT_PREFIX) ? id.slice(SECTION_SORT_PREFIX.length) : null;
}

export function afterDrop(draft: Draft, active: DragItem, over: DragItem | null): Draft {
  if (!over || active.id === over.id || over.section === null) {
    return draft;
  }
  if (active.kind === SECTION_KIND) {
    const from = draft.sections.findIndex((section) => section.id === active.section);
    const to = draft.sections.findIndex((section) => section.id === over.section);
    return from < 0 || to < 0 ? draft : moveSection(draft, draft.sections[from].id, to - from);
  }
  if (over.kind !== WIDGET_KIND) {
    return moveWidget(draft, active.id, over.section, Number.MAX_SAFE_INTEGER);
  }
  const index = widgetsOf(draft, over.section).findIndex((widget) => widget.uid === over.id);
  return moveWidget(draft, active.id, over.section, index);
}

export function byKind(detect: CollisionDetection): CollisionDetection {
  return (args) => {
    const sectionDrag = args.active.data.current?.kind === SECTION_KIND;
    const droppableContainers = args.droppableContainers.filter((container) => (container.data.current?.kind === SECTION_KIND) === sectionDrag);
    return detect({ ...args, droppableContainers });
  };
}

export function keyboardByKind(getter: KeyboardCoordinateGetter): KeyboardCoordinateGetter {
  return (event, args) => {
    const containers = args.context.droppableContainers;
    const sectionDrag = args.context.active?.data.current?.kind === SECTION_KIND;
    const narrowed = {
      getEnabled: () => containers.getEnabled().filter((container) => (container.data.current?.kind === SECTION_KIND) === sectionDrag),
      get: (id: Parameters<typeof containers.get>[0]) => containers.get(id),
    } as unknown as typeof containers;
    return getter(event, { ...args, context: { ...args.context, droppableContainers: narrowed } });
  };
}
