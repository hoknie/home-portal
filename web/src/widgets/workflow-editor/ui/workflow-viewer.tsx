"use client";

import { useTranslations } from "next-intl";
import { type KeyboardEvent, useEffect, useState } from "react";
import { toast } from "sonner";

import { enabledModules, useModules } from "@/entities/module";
import { DeleteWorkflowButton, START_ID, type Workflow, type WorkflowCatalogue, at, namedInputs, parsePath, useRunWorkflow } from "@/entities/workflow";
import { ConflictError, ThrottledError } from "@/shared/api";
import { routes } from "@/shared/config";
import { pushAddress } from "@/shared/lib/navigation";
import { Skeleton } from "@/shared/ui/kit";

import { EditorContext, type Sources } from "../model/editor-context";
import { useEditorState } from "../model/use-editor-state";
import { CanvasArea } from "./canvas-area";
import { PanelRow } from "./resizing/panel-row";
import { StepCard } from "./inspector/step-card";
import { legendDismissed } from "./panels/legend";
import { RunDialog } from "./panels/run-dialog";
import { RunPanel } from "./panels/run-panel";
import { RunsPanel } from "./panels/runs-panel";
import { CloseButton, SideColumn } from "./panels/side-column";
import { ViewToolbar } from "./panels/view-toolbar";

export type WorkflowView = "view" | "history" | "run";

export type WorkflowViewerProps = {
  workflow: Workflow;
  view: WorkflowView;
  run: string | null;
  revision: string | null;
  workflows: Workflow[];
  catalogue: WorkflowCatalogue;
  sources: Omit<Sources, "workflows">;
  tags: string[];
  onRunShown: (workflow: string, run: string) => void;
};

export function WorkflowViewer({ workflow, view, run, revision, workflows, catalogue, sources, tags, onRunShown }: WorkflowViewerProps) {
  const t = useTranslations();
  const runWorkflow = useRunWorkflow();
  const modules = useModules().data?.data;
  const moduleOff = modules !== undefined && !enabledModules(modules).has("workflows");
  const shownRun = view === "run" ? run : null;
  const openRun = (id: string) => pushAddress(routes.workflowRun(workflow.id, id));
  const state = useEditorState({ workflow, selfId: workflow.id, initial: null, workflows, catalogue, sources, tags, readOnly: true, shownRun, openRun });
  const { editor } = state;
  const [legendOpen, setLegendOpen] = useState(() => !legendDismissed());
  const [runOpen, setRunOpen] = useState(false);
  const shownId = state.liveRun?.id ?? null;
  useEffect(() => {
    if (shownId !== null) {
      onRunShown(workflow.id, shownId);
    }
  }, [shownId, onRunShown, workflow.id]);

  const start = async (inputs: Record<string, unknown>) => {
    try {
      const queued = await runWorkflow.mutateAsync({ id: workflow.id, inputs });
      setRunOpen(false);
      openRun(queued.run_id);
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
    } else if (event.key === "?") {
      setLegendOpen(true);
    }
  };


  const chosen = editor.selected && editor.selected !== START_ID ? at(workflow.steps, parsePath(editor.selected).path) : undefined;
  const card = chosen ? (
    <SideColumn title={chosen.label ?? chosen.id} actions={<CloseButton onClose={() => editor.select(null)} />}>
      <div className="p-4">
        <StepCard step={chosen} />
      </div>
    </SideColumn>
  ) : null;

  return (
    <EditorContext.Provider value={editor}>
      <div className="grid gap-3" onKeyDown={keyDown}>
        <ViewToolbar
          workflowId={workflow.id}
          historyOpen={view === "history"}
          running={runWorkflow.isPending}
          runDisabled={moduleOff}
          onRun={() => setRunOpen(true)}
          onLegend={() => setLegendOpen(true)}
          onHistory={() => editor.select(null)}
          remove={<DeleteWorkflowButton workflow={workflow} revision={revision} labelled onDeleted={() => pushAddress(routes.adminWorkflows)} />}
        />
        <PanelRow canvas={<CanvasArea legendOpen={legendOpen} onLegendClosed={() => setLegendOpen(false)} />} narrow={editor.narrow}>
          {view !== "run" && card ? (
            card
          ) : view === "history" ? (
            <SideColumn title={t("workflowEditor.toolbar.history")} closeHref={routes.workflow(workflow.id)}>
              <RunsPanel onChoose={openRun} />
            </SideColumn>
          ) : view === "run" ? (
            <RunPanel
              run={editor.run}
              title={workflow.title}
              stale={editor.stale}
              missing={editor.runMissing}
              historyHref={routes.workflowHistory(workflow.id)}
              closeHref={routes.workflow(workflow.id)}
              empty={<Skeleton data-skeleton="runs" className="h-24" aria-busy="true" />}
            />
          ) : null}
        </PanelRow>
        <RunDialog key={runOpen ? "open" : "closed"} open={runOpen} title={workflow.title} inputs={namedInputs(workflow.inputs)} pending={runWorkflow.isPending} onClose={() => setRunOpen(false)} onRun={(inputs) => void start(inputs)} />
      </div>
    </EditorContext.Provider>
  );
}
