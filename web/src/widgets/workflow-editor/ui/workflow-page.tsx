"use client";

import { useTranslations } from "next-intl";

import { routes } from "@/shared/config";
import { useTrail } from "@/shared/lib/breadcrumbs";

import { useWorkflowData } from "../model/use-workflow-data";
import { WorkflowFrame } from "./workflow-frame";
import { type WorkflowView, WorkflowViewer } from "./workflow-viewer";

export type WorkflowPageProps = { id: string; view: WorkflowView; run: string | null; onRunShown: (workflow: string, run: string) => void };

export function WorkflowPage({ id, view, run, onRunShown }: WorkflowPageProps) {
  const t = useTranslations();
  const trail = useTrail();
  const data = useWorkflowData(id);
  const workflow = data.workflow;
  const name = workflow?.title ?? id;
  const own = { label: name, href: routes.workflow(id), local: true };
  const history = { label: t("breadcrumbs.history"), href: routes.workflowHistory(id), local: true };
  const crumbs =
    view === "view"
      ? trail.of(trail.section("workflows"), { label: name })
      : view === "history"
        ? trail.of(trail.section("workflows"), own, { label: t("breadcrumbs.history") })
        : trail.of(trail.section("workflows"), own, history, { label: t("breadcrumbs.run", { id: run ?? "" }) });
  return (
    <WorkflowFrame crumbs={crumbs} title={name} description={workflow?.description ?? t("workflowEditor.pageDescription")} data={data} missing={workflow === null}>
      {workflow && data.catalogue ? (
        <WorkflowViewer
          key={workflow.id}
          workflow={workflow}
          view={view}
          run={run}
          revision={data.revision}
          workflows={data.all}
          catalogue={data.catalogue}
          sources={data.sources}
          tags={data.tags}
          onRunShown={onRunShown}
        />
      ) : null}
    </WorkflowFrame>
  );
}
