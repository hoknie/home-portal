import { type Draft, moveSection } from "./draft";

export const SECTION_SORT_PREFIX = "sort-section:";

export const SECTION_KIND = "section";

export type DragItem = { id: string; kind: string | null; section: string | null };

export function sectionOfSortable(id: string) {
  return id.startsWith(SECTION_SORT_PREFIX) ? id.slice(SECTION_SORT_PREFIX.length) : null;
}

export function afterDrop(draft: Draft, active: DragItem, over: DragItem | null): Draft {
  if (!over || active.id === over.id || over.section === null || active.kind !== SECTION_KIND) {
    return draft;
  }
  const from = draft.sections.findIndex((section) => section.id === active.section);
  const to = draft.sections.findIndex((section) => section.id === over.section);
  return from < 0 || to < 0 ? draft : moveSection(draft, draft.sections[from].id, to - from);
}
