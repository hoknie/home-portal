"use client";

import { useEffect, useMemo, useState, useSyncExternalStore } from "react";

import { type Run, type Trace, useRun } from "@/entities/automation";
import {
  type Direction,
  type Path,
  START_ID,
  type Step,
  type Target,
  type Workflow,
  type WorkflowCatalogue,
  duplicate,
  filterOffers,
  namedInputs,
  insert,
  move,
  neighbour,
  overlayOf,
  parsePath,
  pathText,
  remove,
  suggestionsAt,
  updateAt,
} from "@/entities/workflow";

import { type Draft, draftOf } from "./draft";
import type { EditorApi, Sources } from "./editor-context";
import { type History, historyOf, recorded, redone, undone } from "./edits/history";
import { problemsFor } from "./checks/problems";
import type { Problems } from "./checks/validation";

export const NARROW_QUERY = "(max-width: 767px)";

function subscribe(onChange: () => void) {
  const query = window.matchMedia(NARROW_QUERY);
  query.addEventListener("change", onChange);
  return () => query.removeEventListener("change", onChange);
}

export function useNarrow() {
  return useSyncExternalStore(
    subscribe,
    () => window.matchMedia(NARROW_QUERY).matches,
    () => false,
  );
}

export type ShownRun = { id: string; steps: Step[] };

export type EditorStateInput = {
  workflow: Workflow | null;
  selfId: string | null;
  initial: Draft | null;
  workflows: Workflow[];
  catalogue: WorkflowCatalogue;
  sources: Omit<Sources, "workflows">;
  tags: string[];
};

export function useEditorState({ workflow, selfId, initial, workflows, catalogue, sources, tags }: EditorStateInput) {
  const [history, setHistory] = useState<History<Draft>>(() => historyOf(initial ?? draftOf(workflow)));
  const [selected, setSelected] = useState<string | null>(null);
  const [palette, setPalette] = useState<Target | null>(null);
  const [server, setServer] = useState<Problems>({});
  const [idFollowsTitle, setIdFollowsTitle] = useState(workflow === null);
  const [shown, setShown] = useState<ShownRun | null>(null);
  const narrow = useNarrow();
  const draft = history.present;
  const live = useRun(shown ? shown.id : null);
  const liveRun: Run | undefined = shown ? live.data : undefined;
  const own = selfId ?? workflow?.id ?? null;
  const others = useMemo(() => workflows.filter((candidate) => candidate.id !== own), [workflows, own]);
  const taken = useMemo(() => others.map((candidate) => candidate.id), [others]);
  const lastRun: Trace | null = liveRun?.trace ?? workflow?.last_run?.trace ?? null;
  const visible = shown !== null && shown.steps === draft.steps;
  const shownTrace = visible ? (liveRun?.trace ?? null) : null;
  const overlay = useMemo(() => overlayOf(shownTrace?.entries ?? []), [shownTrace]);
  const problems = useMemo(
    () => problemsFor({ draft, catalogue, taken, server, secrets: sources.secrets, lastRun, workflow }),
    [draft, catalogue, taken, server, sources.secrets, lastRun, workflow],
  );

  const setDraft = (change: (current: Draft) => Draft, key: string | null = null) => setHistory((current) => recorded(current, change(current.present), key, Date.now()));
  const setSteps = (change: (steps: Step[]) => Step[], key: string | null = null) => setDraft((current) => ({ ...current, steps: change(current.steps) }), key);

  const editor: EditorApi = {
    draft,
    catalogue,
    sources: { ...sources, workflows: others },
    lastRun,
    lastRunId: liveRun?.id ?? workflow?.last_run?.id ?? null,
    problems,
    overlay,
    workflowId: own,
    run: visible ? (liveRun ?? null) : null,
    selected,
    narrow,
    usedBy: workflow?.used_by ?? [],
    tags,
    header: { taken, idFollowsTitle, onIdTyped: (id) => setIdFollowsTitle(workflow === null && id.trim() === "") },
    kindOf: (name) => catalogue.kinds.find((kind) => kind.name === name),
    suggestionsFor: (path: Path, field: string) =>
      suggestionsAt({
        steps: draft.steps,
        inputs: namedInputs(draft.inputs),
        path,
        field,
        catalogue,
        workflows: others,
        eventFields: sources.eventFields,
        secrets: sources.secrets,
        lastRun,
      }),
    filtersFor: (path: Path, field: string, subject: string, chain: string) =>
      filterOffers({ steps: draft.steps, inputs: namedInputs(draft.inputs), path, field, lastRun, catalogue }, subject, chain),
    knownAt: (path: Path, field: string) => ({ steps: draft.steps, inputs: namedInputs(draft.inputs), path, field, lastRun, catalogue }),
    select: setSelected,
    openPalette: setPalette,
    setDraft,
    change: (path, change, key = null) => setSteps((steps) => updateAt(steps, path, change), key),
    insert: (target, step) => {
      setSteps((steps) => insert(steps, target, step));
      setSelected(pathText([...target.owner, { list: target.list, index: target.index }]));
    },
    remove: (path) => {
      setSteps((steps) => remove(steps, path));
      setSelected(null);
    },
    duplicate: (path) => setSteps((steps) => duplicate(steps, path)),
    move: (from, target) => {
      setSteps((steps) => move(steps, from, target));
      setSelected(null);
    },
    showRun: (id) => setShown({ id, steps: draft.steps }),
  };

  const navigate = (direction: Direction) => {
    const current = selected && selected !== START_ID ? parsePath(selected).path : null;
    const next = neighbour(draft.steps, current, direction);
    setSelected(next ? pathText(next) : null);
  };

  useEffect(() => {
    const keyDown = (event: KeyboardEvent) => {
      const typing = event.target instanceof HTMLElement && event.target.closest("input, textarea, select, [contenteditable=true], [role=dialog]") !== null;
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "z" && !typing) {
        event.preventDefault();
        setHistory((current) => (event.shiftKey ? redone(current) : undone(current)));
      }
    };
    window.addEventListener("keydown", keyDown);
    return () => window.removeEventListener("keydown", keyDown);
  }, []);

  return {
    editor,
    history,
    draft,
    palette,
    setPalette,
    setServer,
    shown,
    liveRun,
    setShown,
    navigate,
    undo: () => setHistory(undone),
    redo: () => setHistory(redone),
    replaceDraft: (next: Draft) => setHistory(historyOf(next)),
  };
}
