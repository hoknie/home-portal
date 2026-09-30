"use client";

import { useEffect, useMemo, useRef, useState, useSyncExternalStore } from "react";

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
import { rememberShowValues, showValuesRemembered } from "./values-on-nodes";
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

export type EditorStateInput = {
  workflow: Workflow | null;
  selfId: string | null;
  initial: Draft | null;
  workflows: Workflow[];
  catalogue: WorkflowCatalogue;
  sources: Omit<Sources, "workflows">;
  tags: string[];
  readOnly: boolean;
  shownRun: string | null;
  openRun: ((id: string) => void) | null;
};

export function useEditorState({ workflow, selfId, initial, workflows, catalogue, sources, tags, readOnly, shownRun, openRun }: EditorStateInput) {
  const [history, setHistory] = useState<History<Draft>>(() => historyOf(initial ?? draftOf(workflow)));
  const [selected, setSelected] = useState<string | null>(null);
  const [palette, setPalette] = useState<Target | null>(null);
  const [server, setServer] = useState<Problems>({});
  const [idFollowsTitle, setIdFollowsTitle] = useState(workflow === null);
  const [showValues, setShowValuesState] = useState(showValuesRemembered);
  const [revealed, setRevealed] = useState<{ id: string; at: number } | null>(null);
  const narrow = useNarrow();
  const draft = history.present;
  const live = useRun(shownRun);
  const liveRun: Run | undefined = shownRun && (live.data?.workflow ?? null) === (workflow?.id ?? null) ? live.data : undefined;
  const runMissing = shownRun !== null && (live.isError || (live.data !== undefined && liveRun === undefined));
  const own = selfId ?? workflow?.id ?? null;
  const others = useMemo(() => workflows.filter((candidate) => candidate.id !== own), [workflows, own]);
  const taken = useMemo(() => others.map((candidate) => candidate.id), [others]);
  const lastRecord = useRun(liveRun ? null : (workflow?.last_run?.id ?? null));
  const lastRun: Trace | null = liveRun?.trace ?? lastRecord.data?.trace ?? null;
  const visible = readOnly && shownRun !== null;
  const shownTrace = visible ? (liveRun?.trace ?? null) : null;
  const overlay = useMemo(() => overlayOf(shownTrace?.entries ?? []), [shownTrace]);
  const problems = useMemo(
    () => problemsFor({ draft, catalogue, taken, server, secrets: sources.secrets, lastRun, workflow, portal: sources.portal, events: sources.eventKnowledge }),
    [draft, catalogue, taken, server, sources.secrets, lastRun, workflow, sources.portal, sources.eventKnowledge],
  );

  const setDraft = (change: (current: Draft) => Draft, key: string | null = null) => {
    if (!readOnly) {
      setHistory((current) => recorded(current, change(current.present), key, Date.now()));
    }
  };
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
    runReceivedAt: live.dataUpdatedAt,
    runMissing,
    showValues,
    setShowValues: (show) => {
      rememberShowValues(show);
      setShowValuesState(show);
    },
    readOnly,
    stale: visible && liveRun !== undefined && liveRun.steps_version !== null && workflow !== null && workflow.steps_version !== "" && liveRun.steps_version !== workflow.steps_version,
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
        portal: sources.portal,
      }),
    filtersFor: (path: Path, field: string, subject: string, chain: string) =>
      filterOffers({ steps: draft.steps, inputs: namedInputs(draft.inputs), path, field, lastRun, catalogue, portal: sources.portal }, subject, chain),
    knownAt: (path: Path, field: string) => ({ steps: draft.steps, inputs: namedInputs(draft.inputs), path, field, lastRun, catalogue, portal: sources.portal }),
    select: setSelected,
    openPalette: (target) => {
      if (!readOnly) {
        setPalette(target);
      }
    },
    setDraft,
    change: (path, change, key = null) => setSteps((steps) => updateAt(steps, path, change), key),
    insert: (target, step) => {
      if (readOnly) {
        return;
      }
      setSteps((steps) => insert(steps, target, step));
      setSelected(pathText([...target.owner, { list: target.list, index: target.index }]));
    },
    remove: (path) => {
      if (readOnly) {
        return;
      }
      setSteps((steps) => remove(steps, path));
      setSelected(null);
    },
    duplicate: (path) => setSteps((steps) => duplicate(steps, path)),
    move: (from, target) => {
      if (readOnly) {
        return;
      }
      setSteps((steps) => move(steps, from, target));
      setSelected(null);
    },
    openRun,
    revealed,
    reveal: (id) => {
      setSelected(id);
      setRevealed({ id, at: Date.now() });
    },
  };

  const navigate = (direction: Direction) => {
    const current = selected && selected !== START_ID ? parsePath(selected).path : null;
    const next = neighbour(draft.steps, current, direction);
    setSelected(next ? pathText(next) : null);
  };

  const readOnlyRef = useRef(readOnly);
  useEffect(() => {
    readOnlyRef.current = readOnly;
  }, [readOnly]);

  useEffect(() => {
    const keyDown = (event: KeyboardEvent) => {
      const typing = event.target instanceof HTMLElement && event.target.closest("input, textarea, select, [contenteditable=true], [role=dialog]") !== null;
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "z" && !typing && !readOnlyRef.current) {
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
    liveRun,
    navigate,
    undo: () => (readOnly ? undefined : setHistory(undone)),
    redo: () => (readOnly ? undefined : setHistory(redone)),
    replaceDraft: (next: Draft) => setHistory(historyOf(next)),
  };
}
