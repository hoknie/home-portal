"use client";

import { useState } from "react";

export type EditorRevision = {
  revision: string | null;
  latest: string | null;
  adopt: (next: string | null) => void;
  catchUp: () => string | null;
};

export function useEditorRevision(latest: string | null): EditorRevision {
  const [held, setHeld] = useState<string | null>(latest);
  if (held === null && latest !== null) {
    setHeld(latest);
  }
  return {
    revision: held ?? latest,
    latest,
    adopt: setHeld,
    catchUp: () => {
      setHeld(latest);
      return latest;
    },
  };
}
