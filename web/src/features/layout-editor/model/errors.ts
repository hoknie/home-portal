import type { FieldError } from "@/shared/api";

import type { Draft } from "./draft";

export type PlacedErrors = { widgets: Record<string, string[]>; sections: Record<string, string[]>; other: string[] };

export function placeErrors(errors: FieldError[], draft: Draft, sent: Draft): PlacedErrors {
  const placed: PlacedErrors = { widgets: {}, sections: {}, other: [] };
  const order = sent.sections.flatMap((section) => sent.widgets.filter((widget) => widget.section === section.id));
  for (const error of errors) {
    const text = `${error.field}: ${error.message}`;
    const widget = /^widgets\[(\d+)\]/.exec(error.field);
    const section = /^sections\[(\d+)\]/.exec(error.field);
    const uid = widget ? order[Number(widget[1])]?.uid : undefined;
    const id = section ? sent.sections[Number(section[1])]?.id : undefined;
    if (uid && draft.widgets.some((candidate) => candidate.uid === uid)) {
      placed.widgets[uid] = [...(placed.widgets[uid] ?? []), text];
    } else if (id) {
      placed.sections[id] = [...(placed.sections[id] ?? []), text];
    } else {
      placed.other.push(text);
    }
  }
  return placed;
}
