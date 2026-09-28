"use client";

import { useTranslations } from "next-intl";
import { type KeyboardEvent, useEffect, useState } from "react";
import { toast } from "sonner";

import {
  START_ID,
  type Workflow,
  type WorkflowCatalogue,
  idsOf,
  namedInputs,
  newStep,
  parsePath,
  pathText,
  useRunWorkflow,
  useSaveWorkflow,
} from "@/entities/workflow";
import { ConflictError, ThrottledError, ValidationError } from "@/shared/api";
import { useLeaveGuard } from "@/shared/lib/leave-guard";
import { ErrorNotice } from "@/shared/ui/error-notice";

import { blocksToOpen, problemsFromServer } from "../model/checks/placing";
import { type Problem, blocking } from "../model/checks/problems";
import { type Draft, requestOf, sameDraft, sameSteps } from "../model/draft";
import { EditorContext, type Sources } from "../model/editor-context";
import { useEditorState } from "../model/use-editor-state";
import { CanvasLoader } from "./canvas/canvas-loader";
import { fieldId } from "./inspector/template-field";
import { Inspector } from "./inspector/inspector";
import { Legend, legendDismissed, rememberLegend } from "./panels/legend";
import { Palette } from "./panels/palette";
import { ProblemsPanel } from "./panels/problems-panel";
import { RunPanel } from "./panels/run-panel";
import { RunDialog } from "./panels/run-dialog";
import { RunsPanel } from "./panels/runs-panel";
import { Toolbar } from "./panels/toolbar";

export type WorkflowEditorProps = {
  workflow: Workflow | null;
  initial?: Draft | null;
  revision: string | null;
  workflows: Workflow[];
  catalogue: WorkflowCatalogue;
  sources: Omit<Sources, "workflows">;
  tags: string[];
  onSaved: () => void;
  onConflict: () => void;
};

const HEADER_FIELDS: Record<string, string> = { title: "workflow-title", id: "workflow-id", timeout_seconds: "workflow-timeout" };

export function WorkflowEditor({ workflow, initial = null, revision, workflows, catalogue, sources, tags, onSaved, onConflict }: WorkflowEditorProps) {
  const t = useTranslations();
  const save = useSaveWorkflow();
  const runWorkflow = useRunWorkflow();
  const [identity, setIdentity] = useState<string | null>(workflow?.id ?? null);
  const state = useEditorState({ workflow, selfId: identity, initial, workflows, catalogue, sources, tags });
  const { editor, draft } = state;
  const [base, setBase] = useState<Draft>(draft);
  const [savedRevision, setSavedRevision] = useState<string | null>(null);
  const [conflict, setConflict] = useState(false);
  const [leaving, setLeaving] = useState(false);
  const [problemsOpen, setProblemsOpen] = useState(false);
  const [runsOpen, setRunsOpen] = useState(false);
  const [legendOpen, setLegendOpen] = useState(() => !legendDismissed());
  const [runOpen, setRunOpen] = useState(false);
  const [focusRequest, setFocusRequest] = useState<string | null>(null);
  const dirty = !sameDraft(draft, base);
  useLeaveGuard(dirty && !leaving, t("workflowEditor.leave"));
  useEffect(() => {
    if (focusRequest) {
      document.getElementById(focusRequest)?.focus();
    }
  }, [focusRequest, editor.selected]);
  const errors = blocking(editor.problems).length;
  const warnings = editor.problems.length - errors;

  const choose = (problem: Problem) => {
    const { path, field } = parsePath(problem.at);
    if (path.length === 0) {
      editor.select(problem.at === "" ? null : START_ID);
      setFocusRequest(HEADER_FIELDS[problem.at] ?? null);
      return;
    }
    editor.select(pathText(path));
    setFocusRequest(field ? fieldId(path, field) : null);
  };

  const store = async () => {
    setConflict(false);
    if (errors > 0) {
      setProblemsOpen(true);
      toast.error(t("workflowEditor.fixProblems"));
      return false;
    }
    try {
      const saved = await save.mutateAsync({ id: identity, body: requestOf(draft), revision: savedRevision ?? revision });
      setBase(draft);
      setIdentity(saved.data.id);
      setSavedRevision(saved.revision);
      state.setServer({});
      toast.success(t(identity ? "workflowEditor.saved" : "workflowEditor.created"));
      return true;
    } catch (error) {
      if (error instanceof ValidationError) {
        const found = problemsFromServer(error.fields);
        state.setServer(found);
        setProblemsOpen(true);
        const first = blocksToOpen(found)[0];
        if (first) {
          editor.select(first);
        }
      } else if (error instanceof ConflictError) {
        setConflict(true);
        setSavedRevision(null);
        onConflict();
      } else {
        toast.error(t("errors.generic"));
      }
      return false;
    }
  };

  const saveAndLeave = async () => {
    if (editor.readOnly) {
      return;
    }
    if (await store()) {
      setLeaving(true);
      onSaved();
    }
  };

  const saveAndRun = async () => {
    if ((dirty || identity === null) && !(await store())) {
      return;
    }
    setRunOpen(true);
  };

  const run = async (inputs: Record<string, unknown>) => {
    if (!identity) {
      return;
    }
    try {
      const queued = await runWorkflow.mutateAsync({ id: identity, inputs });
      editor.showRun(queued.run_id);
      setRunOpen(false);
    } catch (error) {
      toast.error(t(error instanceof ThrottledError ? "workflows.runThrottled" : error instanceof ConflictError ? "workflows.runRefused" : "workflows.runFailed", { seconds: error instanceof ThrottledError ? error.retryAfterSeconds : 0 }));
    }
  };

  const keyDown = (event: KeyboardEvent) => {
    if (event.target instanceof HTMLElement && event.target.closest("input, textarea, select, [role=dialog], [role=menu]")) {
      return;
    }
    const moves: Record<string, "next" | "previous" | "into" | "out"> = { ArrowDown: "next", ArrowUp: "previous", ArrowRight: "into", ArrowLeft: "out" };
    if (moves[event.key]) {
      event.preventDefault();
      state.navigate(moves[event.key]);
    } else if ((event.key === "Delete" || event.key === "Backspace") && !editor.readOnly && editor.selected && editor.selected !== START_ID) {
      event.preventDefault();
      editor.remove(parsePath(editor.selected).path);
    } else if (event.key === "?") {
      setLegendOpen(true);
    }
  };

  return (
    <EditorContext.Provider value={editor}>
      <form
        className="grid gap-3"
        noValidate
        onKeyDown={keyDown}
        onSubmit={(event) => {
          event.preventDefault();
          void saveAndLeave();
        }}
      >
        {conflict ? <ErrorNotice title={t("errors.conflict")} description={t("workflowEditor.conflictKept")} /> : null}
        <div className="relative">
          <Toolbar
            canUndo={state.history.past.length > 0}
            canRedo={state.history.future.length > 0}
            errors={errors}
            warnings={warnings}
            problemsOpen={problemsOpen}
            runsOpen={runsOpen}
            saving={save.isPending}
            onUndo={state.undo}
            onRedo={state.redo}
            onProblems={() => {
              setRunsOpen(false);
              setProblemsOpen((open) => !open);
            }}
            onRuns={() => {
              setProblemsOpen(false);
              setRunsOpen((open) => !open);
            }}
            onLegend={() => setLegendOpen(true)}
            onSaveAndRun={() => void saveAndRun()}
            mode={editor.mode}
            onMode={editor.setMode}
          />
          {problemsOpen ? (
            <div className="glass-panel absolute top-full left-0 z-30 mt-2 w-[28rem] max-w-full rounded-xl shadow-lg">
              <ProblemsPanel onChoose={choose} />
            </div>
          ) : null}
          {runsOpen ? (
            <div className="glass-panel absolute top-full left-0 z-30 mt-2 w-[28rem] max-w-full rounded-xl shadow-lg">
              <RunsPanel
                onChoose={(id) => {
                  editor.showRun(id);
                  setRunsOpen(false);
                }}
              />
            </div>
          ) : null}
        </div>
        <div className="flex h-[calc(100dvh-13rem)] min-h-[34rem] flex-col gap-3 md:flex-row">
          <div className="relative min-w-0 flex-1 overflow-hidden rounded-2xl border border-glass-edge bg-background/40">
            <CanvasLoader />
            {legendOpen ? (
              <div className="absolute top-3 left-3 z-20">
                <Legend
                  onDismiss={() => {
                    rememberLegend(true);
                    setLegendOpen(false);
                  }}
                />
              </div>
            ) : null}
          </div>
          {editor.readOnly ? (
            <RunPanel run={editor.run} title={draft.title} stale={editor.stale || (dirty && !sameSteps(draft, base))} onHide={() => editor.setMode("edit")} />
          ) : editor.selected ? (
            <Inspector />
          ) : null}
        </div>
        <Palette
          target={editor.readOnly ? null : state.palette}
          onClose={() => state.setPalette(null)}
          onChoose={(kind, target) => {
            state.setPalette(null);
            editor.insert(target, newStep(kind, idsOf(draft.steps)));
          }}
        />
        <RunDialog open={runOpen} title={draft.title} inputs={namedInputs(draft.inputs)} pending={runWorkflow.isPending} onClose={() => setRunOpen(false)} onRun={(inputs) => void run(inputs)} />
      </form>
    </EditorContext.Provider>
  );
}
