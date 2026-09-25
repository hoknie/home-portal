import type { Dashboard, LayoutRequest, LayoutWidgetRequest } from "@/entities/dashboard";
import type { Section } from "@/shared/api";

export type DraftWidget = LayoutWidgetRequest & { uid: string };

export type Draft = { sections: Section[]; widgets: DraftWidget[] };

export const IMPLICIT_SECTION = "main";

export function fromLayout(layout: Dashboard): Draft {
  const sections = layout.sections.length > 0 ? layout.sections : [{ id: IMPLICIT_SECTION, title: null }];
  const known = new Set(sections.map((section) => section.id));
  const widgets = layout.widgets.map((widget, index) => ({
    uid: widget.key || `#${index}`,
    key: widget.key || null,
    type: widget.type,
    id: widget.id,
    title: widget.title,
    settings: widget.settings,
    environments: widget.environments,
    public: widget.public,
    section: widget.section && known.has(widget.section) ? widget.section : sections[0].id,
    size: widget.size,
  }));
  return normalized({ sections, widgets });
}

export function toRequest(draft: Draft): LayoutRequest {
  return {
    sections: draft.sections,
    widgets: normalized(draft).widgets.map((widget) => ({
      key: widget.key,
      type: widget.type,
      id: widget.id,
      title: widget.title,
      settings: widget.settings,
      environments: widget.environments,
      public: widget.public,
      section: widget.section,
      size: widget.size,
    })),
  };
}

export function sameDraft(left: Draft, right: Draft) {
  return JSON.stringify(toRequest(left)) === JSON.stringify(toRequest(right));
}

export function widgetsOf(draft: Draft, section: string) {
  return draft.widgets.filter((widget) => widget.section === section);
}

export function moveWidget(draft: Draft, uid: string, section: string, index: number): Draft {
  const moving = draft.widgets.find((widget) => widget.uid === uid);
  if (!moving || !draft.sections.some((candidate) => candidate.id === section)) {
    return draft;
  }
  const rest = draft.widgets.filter((widget) => widget.uid !== uid);
  const target = rest.filter((widget) => widget.section === section);
  const clamped = Math.max(0, Math.min(index, target.length));
  target.splice(clamped, 0, { ...moving, section });
  const widgets = draft.sections.flatMap((candidate) =>
    candidate.id === section ? target : rest.filter((widget) => widget.section === candidate.id),
  );
  return { ...draft, widgets };
}

export function updateWidget(draft: Draft, uid: string, patch: Partial<LayoutWidgetRequest>): Draft {
  if (patch.section && patch.section !== draft.widgets.find((widget) => widget.uid === uid)?.section) {
    const moved = moveWidget(draft, uid, patch.section, Number.MAX_SAFE_INTEGER);
    return updateWidget(moved, uid, { ...patch, section: undefined });
  }
  const clean = Object.fromEntries(Object.entries(patch).filter(([, value]) => value !== undefined));
  return { ...draft, widgets: draft.widgets.map((widget) => (widget.uid === uid ? { ...widget, ...clean } : widget)) };
}

export function addWidget(draft: Draft, type: string, section: string): Draft {
  const uid = freshUid(draft);
  const widget: DraftWidget = {
    uid,
    key: null,
    type,
    id: null,
    title: null,
    settings: {},
    environments: null,
    public: false,
    section,
    size: "full",
  };
  return moveWidget({ ...draft, widgets: [...draft.widgets, widget] }, uid, section, Number.MAX_SAFE_INTEGER);
}

export function removeWidget(draft: Draft, uid: string): Draft {
  return { ...draft, widgets: draft.widgets.filter((widget) => widget.uid !== uid) };
}

export function addSection(draft: Draft, title: string | null): Draft {
  const taken = new Set(draft.sections.map((section) => section.id));
  let number = draft.sections.length + 1;
  while (taken.has(`section-${number}`)) {
    number += 1;
  }
  return { ...draft, sections: [...draft.sections, { id: `section-${number}`, title }] };
}

export function renameSection(draft: Draft, id: string, title: string): Draft {
  const trimmed = title.trim();
  return {
    ...draft,
    sections: draft.sections.map((section) => (section.id === id ? { ...section, title: trimmed === "" ? null : trimmed } : section)),
  };
}

export function moveSection(draft: Draft, id: string, delta: number): Draft {
  const from = draft.sections.findIndex((section) => section.id === id);
  const to = from + delta;
  if (from < 0 || to < 0 || to >= draft.sections.length) {
    return draft;
  }
  const sections = [...draft.sections];
  const [moved] = sections.splice(from, 1);
  sections.splice(to, 0, moved);
  return normalized({ ...draft, sections });
}

export function removeSection(draft: Draft, id: string): Draft {
  if (draft.sections.length <= 1 || widgetsOf(draft, id).length > 0) {
    return draft;
  }
  return { ...draft, sections: draft.sections.filter((section) => section.id !== id) };
}

function normalized(draft: Draft): Draft {
  return { ...draft, widgets: draft.sections.flatMap((section) => widgetsOf(draft, section.id)) };
}

function freshUid(draft: Draft) {
  let number = draft.widgets.length + 1;
  while (draft.widgets.some((widget) => widget.uid === `new-${number}`)) {
    number += 1;
  }
  return `new-${number}`;
}
