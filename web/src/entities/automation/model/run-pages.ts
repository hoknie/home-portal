"use client";

import { useState } from "react";

export type RunPages = {
  before: string | null;
  page: number;
  older: (next: string) => void;
  newer: () => void;
  newest: () => void;
};

export function useRunPages(scope = ""): RunPages {
  const [state, setState] = useState<{ scope: string; stack: string[] }>({ scope, stack: [] });
  const stack = state.scope === scope ? state.stack : [];
  return {
    before: stack.at(-1) ?? null,
    page: stack.length + 1,
    older: (next) => setState({ scope, stack: [...stack, next] }),
    newer: () => setState({ scope, stack: stack.slice(0, -1) }),
    newest: () => setState({ scope, stack: [] }),
  };
}
