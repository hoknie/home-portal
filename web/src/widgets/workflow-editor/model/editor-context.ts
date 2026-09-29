import { createContext, useContext } from "react";

import type { Run, Trace } from "@/entities/automation";
import type { HeaderProblem, ScriptArgument } from "@/entities/script";
import type { FilterOffer, Kind, KnownContext, PortalValues, Overlay, Path, Step, Suggestion, Target, Usage, Workflow, WorkflowCatalogue } from "@/entities/workflow";

import type { Draft } from "./draft";
import type { Problem } from "./checks/problems";

export type Sources = {
  workflows: Workflow[];
  services: { id: string; name: string }[];
  states: string[];
  scripts: { path: string; runnable: boolean; problem: string | null; description?: string | null; arguments?: ScriptArgument[]; argument_problems?: HeaderProblem[] }[];
  secrets: { name: string; set: boolean }[];
  channels: { name: string; readiness: string }[];
  portal: PortalValues | null;
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
  runReceivedAt: number;
  runMissing: boolean;
  showValues: boolean;
  setShowValues: (show: boolean) => void;
  readOnly: boolean;
  stale: boolean;
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
  openRun: ((id: string) => void) | null;
  revealed: { id: string; at: number } | null;
  reveal: (id: string) => void;
};

export const EditorContext = createContext<EditorApi | null>(null);

export function useEditor(): EditorApi {
  const editor = useContext(EditorContext);
  if (editor === null) {
    throw new Error("useEditor outside the workflow editor");
  }
  return editor;
}
