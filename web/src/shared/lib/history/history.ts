export const KEPT_STEPS = 50;

export type Step<Value> = { value: Value; label: string };

export type History<Value> = { past: Step<Value>[]; present: Value; future: Step<Value>[] };

export function begin<Value>(present: Value): History<Value> {
  return { past: [], present, future: [] };
}

export function pushed<Value>(history: History<Value>, next: Value, label: string, same: (left: Value, right: Value) => boolean): History<Value> {
  if (same(history.present, next)) {
    return history;
  }
  return { past: [...history.past, { value: history.present, label }].slice(-KEPT_STEPS), present: next, future: [] };
}

export function undone<Value>(history: History<Value>): History<Value> {
  const previous = history.past.at(-1);
  if (previous === undefined) {
    return history;
  }
  return {
    past: history.past.slice(0, -1),
    present: previous.value,
    future: [{ value: history.present, label: previous.label }, ...history.future],
  };
}

export function redone<Value>(history: History<Value>): History<Value> {
  const [next, ...rest] = history.future;
  if (next === undefined) {
    return history;
  }
  return { past: [...history.past, { value: history.present, label: next.label }].slice(-KEPT_STEPS), present: next.value, future: rest };
}

export function nextUndo<Value>(history: History<Value>) {
  return history.past.at(-1)?.label ?? null;
}

export function nextRedo<Value>(history: History<Value>) {
  return history.future[0]?.label ?? null;
}

export function isUndoKey(event: Pick<KeyboardEvent, "key" | "ctrlKey" | "metaKey" | "shiftKey">) {
  return (event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "z" && !event.shiftKey;
}

export function isRedoKey(event: Pick<KeyboardEvent, "key" | "ctrlKey" | "metaKey" | "shiftKey">) {
  return (event.ctrlKey || event.metaKey) && ((event.key.toLowerCase() === "z" && event.shiftKey) || event.key.toLowerCase() === "y");
}

export function typingIn(target: EventTarget | null) {
  if (!(target instanceof HTMLElement)) {
    return false;
  }
  return target.isContentEditable || ["INPUT", "TEXTAREA", "SELECT"].includes(target.tagName);
}
