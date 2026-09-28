export const MOST_ENTRIES = 100;
export const COALESCE_MILLISECONDS = 600;

export type History<T> = { past: T[]; present: T; future: T[]; lastKey: string | null; lastAt: number };

export function historyOf<T>(present: T): History<T> {
  return { past: [], present, future: [], lastKey: null, lastAt: 0 };
}

export function recorded<T>(history: History<T>, next: T, key: string | null, now: number): History<T> {
  if (next === history.present) {
    return history;
  }
  if (key !== null && key === history.lastKey && now - history.lastAt < COALESCE_MILLISECONDS) {
    return { ...history, present: next, future: [], lastAt: now };
  }
  return {
    past: [...history.past, history.present].slice(-MOST_ENTRIES),
    present: next,
    future: [],
    lastKey: key,
    lastAt: now,
  };
}

export function undone<T>(history: History<T>): History<T> {
  const previous = history.past.at(-1);
  if (previous === undefined) {
    return history;
  }
  return { past: history.past.slice(0, -1), present: previous, future: [history.present, ...history.future], lastKey: null, lastAt: 0 };
}

export function redone<T>(history: History<T>): History<T> {
  const [next, ...rest] = history.future;
  if (next === undefined) {
    return history;
  }
  return { past: [...history.past, history.present].slice(-MOST_ENTRIES), present: next, future: rest, lastKey: null, lastAt: 0 };
}
