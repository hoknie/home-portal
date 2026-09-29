"use client";

import { useTranslations } from "next-intl";

import { useTrail } from "@/shared/lib/breadcrumbs";
import { pushAddress } from "@/shared/lib/navigation";
import { routes } from "@/shared/config";
import { slugOf, uniqueId } from "@/shared/lib/slug";

import { localizedTemplate } from "../model/localize";
import { useWorkflowData } from "../model/use-workflow-data";
import { WorkflowEditor } from "./workflow-editor";
import { WorkflowFrame } from "./workflow-frame";

export type WorkflowEditorScreenProps = { mode: "new" | "edit"; id: string | null; template: string | null; lastShownRun: string | null };

export function WorkflowEditorScreen({ mode, id, template, lastShownRun }: WorkflowEditorScreenProps) {
  const t = useTranslations();
  const help = useTranslations("workflowHelp");
  const trail = useTrail();
  const data = useWorkflowData(mode === "edit" ? id : null);
  const workflow = data.workflow;
  const crumbs =
    mode === "new"
      ? trail.of(trail.section("workflows"), { label: t("breadcrumbs.new.workflow") })
      : trail.of(trail.section("workflows"), { label: workflow?.title ?? id ?? "", href: routes.workflow(id ?? ""), local: true }, { label: t("breadcrumbs.edit") });
  const templateName = mode === "new" ? template : null;
  const initialFor = () => {
    const found = localizedTemplate(templateName, { has: (key) => help.has(key as "kinds.if.name"), text: (key) => help(key as "kinds.if.name") });
    const taken = data.all.map((candidate) => candidate.id);
    return found ? { ...found, id: found.title ? uniqueId(slugOf(found.title), taken) : "" } : null;
  };
  return (
    <WorkflowFrame
      crumbs={crumbs}
      title={t(mode === "new" ? "workflowEditor.addTitle" : "workflowEditor.editTitle")}
      description={t("workflowEditor.pageDescription")}
      data={data}
      missing={mode === "edit" && data.workflow === null}
    >
      {data.catalogue ? (
        <WorkflowEditor
          key={workflow?.id ?? `new-${templateName ?? ""}`}
          workflow={workflow}
          initial={initialFor()}
          revision={data.revision}
          workflows={data.all}
          catalogue={data.catalogue}
          sources={data.sources}
          tags={data.tags}
          lastShownRun={lastShownRun}
          onSaved={(saved) => pushAddress(routes.workflow(saved))}
          onCancel={() => pushAddress(mode === "new" || id === null ? routes.adminWorkflows : routes.workflow(id))}
          onConflict={data.refetch}
        />
      ) : null}
    </WorkflowFrame>
  );
}
