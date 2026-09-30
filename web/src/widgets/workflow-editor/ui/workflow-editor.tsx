"use client";

import { useTranslations } from "next-intl";
import { type KeyboardEvent, useEffect, useState } from "react";
import { toast } from "sonner";

import {
  START_ID,
  type Workflow,
  type WorkflowCatalogue,
  idsOf,
  newStep,
  parsePath,
  pathText,
  useSaveWorkflow,
} from "@/entities/workflow";
import { useCan } from "@/entities/session";
import { ConflictError, ValidationError } from "@/shared/api";
import { useEditorRevision } from "@/shared/lib/editor-revision";
import { useLeaveGuard } from "@/shared/lib/leave-guard";
import { ConflictNotice } from "@/shared/ui/conflict-notice";

import { blocksToOpen, problemsFromServer } from "../model/checks/placing";
import { type Problem, blocking } from "../model/checks/problems";
import { type Draft, draftOf, requestOf, sameDraft } from "../model/draft";
import { filledSteps } from "../model/edits/filling";
import { EditorContext, type Sources } from "../model/editor-context";
import { useEditorState } from "../model/use-editor-state";
import { CanvasArea } from "./canvas-area";
import { PanelRow } from "./resizing/panel-row";
import { fieldId } from "./inspector/template-field";
import { Inspector } from "./inspector/inspector";
import { EditToolbar } from "./panels/edit-toolbar";
import { legendDismissed } from "./panels/legend";
import { Palette } from "./panels/palette";
import { ProblemsPanel } from "./panels/problems-panel";
import { SideColumn } from "./panels/side-column";

export type WorkflowEditorProps = {
  workflow: Workflow | null;
  initial?: Draft | null;
  revision: string | null;
  workflows: Workflow[];
  catalogue: WorkflowCatalogue;
  sources: Omit<Sources, "workflows">;
  tags: string[];
  lastShownRun?: string | null;
  onSaved: (id: string) => void;
  onCancel: () => void;
  onConflict: () => void;
};

const HEADER_FIELDS: Record<string, string> = { title: "workflow-title", id: "workflow-id", timeout_seconds: "workflow-timeout" };

export function WorkflowEditor({ workflow, initial = null, revision, workflows, catalogue, sources, tags, lastShownRun = null, onSaved, onCancel, onConflict }: WorkflowEditorProps) {
  const t = useTranslations();
  const can = useCan();
  const save = useSaveWorkflow();
  const [identity, setIdentity] = useState<string | null>(workflow?.id ?? null);
  const state = useEditorState({ workflow, selfId: identity, initial, workflows, catalogue, sources, tags, readOnly: false, shownRun: lastShownRun, openRun: null });
  const { editor, draft } = state;
  const [base, setBase] = useState<Draft>(draft);
  const held = useEditorRevision(revision);
  const [conflict, setConflict] = useState(false);
  const [savedTo, setSavedTo] = useState<string | null>(null);
  const [problemsOpen, setProblemsOpen] = useState(false);
  const [legendOpen, setLegendOpen] = useState(() => !legendDismissed());
  const [focusRequest, setFocusRequest] = useState<string | null>(null);
  const dirty = !sameDraft(draft, base);
  useLeaveGuard(dirty && savedTo === null, t("workflowEditor.leave"));
  useEffect(() => {
    if (savedTo !== null) {
      onSaved(savedTo);
    }
  }, [savedTo, onSaved]);
  useEffect(() => {
    if (focusRequest) {
      document.getElementById(focusRequest)?.focus();
    }
  }, [focusRequest, editor.selected]);
  const errors = blocking(editor.problems).length;
  const warnings = editor.problems.length - errors;

  const choose = (problem: Problem) => {
    setProblemsOpen(false);
    const { path, field } = parsePath(problem.at);
    if (path.length === 0) {
      editor.select(problem.at === "" ? null : START_ID);
      setFocusRequest(HEADER_FIELDS[problem.at] ?? null);
      return;
    }
    editor.select(pathText(path));
    setFocusRequest(field ? fieldId(path, field) : null);
  };

  const store = async (at: string | null = held.revision): Promise<string | null> => {
    setConflict(false);
    if (errors > 0) {
      setProblemsOpen(true);
      toast.error(t("workflowEditor.fixProblems"));
      return null;
    }
    try {
      const filled: Draft = { ...draft, steps: filledSteps(draft.steps) };
      const saved = await save.mutateAsync({ id: identity, body: requestOf(filled), revision: at });
      if (!sameDraft(filled, draft)) {
        state.replaceDraft(filled);
      }
      setBase(filled);
      setIdentity(saved.data.id);
      held.adopt(saved.revision);
      state.setServer({});
      toast.success(t(identity ? "workflowEditor.saved" : "workflowEditor.created"));
      return saved.data.id;
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
        onConflict();
      } else {
        toast.error(t("errors.generic"));
      }
      return null;
    }
  };

  const reload = () => {
    const current = draftOf(workflow);
    held.catchUp();
    state.replaceDraft(current);
    setBase(current);
    state.setServer({});
    setConflict(false);
  };

  const saveAndLeave = async (at: string | null = held.revision) => {
    const saved = await store(at);
    if (saved !== null) {
      setSavedTo(saved);
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
    } else if ((event.key === "Delete" || event.key === "Backspace") && editor.selected && editor.selected !== START_ID) {
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
        {conflict ? <ConflictNotice pending={held.latest === held.revision || save.isPending} onReload={reload} onOverwrite={() => void saveAndLeave(held.catchUp())} /> : null}
        <EditToolbar
          canUndo={state.history.past.length > 0}
          canRedo={state.history.future.length > 0}
          errors={errors}
          warnings={warnings}
          problemsOpen={problemsOpen}
          saving={save.isPending}
          canSave={can("workflows", workflow === null ? "create" : "update")}
          onUndo={state.undo}
          onRedo={state.redo}
          onProblems={() => setProblemsOpen((open) => !open)}
          onLegend={() => setLegendOpen(true)}
          onCancel={onCancel}
        />
        <PanelRow canvas={<CanvasArea legendOpen={legendOpen} onLegendClosed={() => setLegendOpen(false)} />} narrow={editor.narrow}>
          {problemsOpen ? (
            <SideColumn title={t("workflowEditor.problemsPanel.title")}>
              <ProblemsPanel onChoose={choose} />
            </SideColumn>
          ) : editor.selected ? (
            <Inspector />
          ) : null}
        </PanelRow>
        <Palette
          target={state.palette}
          onClose={() => state.setPalette(null)}
          onChoose={(kind, target) => {
            state.setPalette(null);
            editor.insert(target, newStep(kind, idsOf(draft.steps)));
          }}
        />
      </form>
    </EditorContext.Provider>
  );
}
