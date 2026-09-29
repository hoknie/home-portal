"use client";

import { useCallback, useState } from "react";

import { TEMPLATE_PARAMETER, workflowAddressOf } from "@/entities/workflow";
import { useAddress } from "@/shared/lib/navigation";
import { Skeleton } from "@/shared/ui/primitives";
import { WorkflowEditorScreen, WorkflowPage } from "@/widgets/workflow-editor";
import { WorkflowsScreen } from "@/widgets/workflows";

type Shown = { workflow: string; run: string };

export function WorkflowsRoute() {
  const { path } = useAddress();
  const [shown, setShown] = useState<Shown | null>(null);
  const remember = useCallback((workflow: string, run: string) => {
    setShown((current) => (current?.workflow === workflow && current.run === run ? current : { workflow, run }));
  }, []);
  if (path === null) {
    return <Skeleton className="h-96 w-full" aria-busy="true" />;
  }
  const address = workflowAddressOf(path);
  switch (address.kind) {
    case "list":
      return <WorkflowsScreen />;
    case "new":
      return <WorkflowEditorScreen key="new" mode="new" id={null} template={new URLSearchParams(window.location.search).get(TEMPLATE_PARAMETER)} lastShownRun={null} />;
    case "edit":
      return <WorkflowEditorScreen key={`edit-${address.id}`} mode="edit" id={address.id} template={null} lastShownRun={shown?.workflow === address.id ? shown.run : null} />;
    case "run":
      return <WorkflowPage key={`page-${address.id}`} id={address.id} view="run" run={address.run} onRunShown={remember} />;
    case "history":
      return <WorkflowPage key={`page-${address.id}`} id={address.id} view="history" run={null} onRunShown={remember} />;
    default:
      return <WorkflowPage key={`page-${address.id}`} id={address.id} view="view" run={null} onRunShown={remember} />;
  }
}
