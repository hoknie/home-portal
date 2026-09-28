import { createContext, useContext } from "react";

import type { Run, Trace } from "@/entities/automation";
import type { FilterOffer, Kind, KnownContext, Overlay, Path, Step, Suggestion, Target, Usage, Workflow, WorkflowCatalogue } from "@/entities/workflow";

import type { Draft } from "./draft";
import type { Problem } from "./checks/problems";

export type Sources = {
  workflows: Workflow[];
  services: { id: string; name: string }[];
  states: string[];
  scripts: { path: string; runnable: boolean; problem: string | null }[];
  secrets: { name: string; set: boolean }[];
  channels: { name: string; readiness: string }[];
  automations: { id: string; title: string; event: string; enabled: boolean }[];
  eventFields: { name: string; sample: string }[];
};

export type EditorApi = {
  draft: Draft;
  catalogue: WorkflowCatalogue;
  sources: Sources;
  lastRun: Trace | null;
  lastRunId: string | null;
  problems: Problem[];
  overlay: Overlay;
  workflowId: string | null;
  run: Run | null;
  selected: string | null;
  narrow: boolean;
  usedBy: Usage[];
  tags: string[];
  header: { taken: string[]; idFollowsTitle: boolean; onIdTyped: (id: string) => void };
  kindOf: (name: string) => Kind | undefined;
  suggestionsFor: (path: Path, field: string) => Suggestion[];
  filtersFor: (path: Path, field: string, subject: string, chain: string) => FilterOffer[];
  knownAt: (path: Path, field: string) => KnownContext;
  select: (id: string | null) => void;
  openPalette: (target: Target) => void;
  setDraft: (change: (draft: Draft) => Draft, key?: string | null) => void;
  change: (path: Path, change: (step: Step) => Step, key?: string | null) => void;
  insert: (target: Target, step: Step) => void;
  remove: (path: Path) => void;
  duplicate: (path: Path) => void;
  move: (from: Path, target: Target) => void;
  showRun: (id: string) => void;
};

export const EditorContext = createContext<EditorApi | null>(null);

export function useEditor(): EditorApi {
  const editor = useContext(EditorContext);
  if (editor === null) {
    throw new Error("useEditor outside the workflow editor");
  }
  return editor;
}
