import { Suspense } from "react";

import { WorkflowEditorScreen } from "@/widgets/workflow-editor";

export default function NewWorkflowPage() {
  return (
    <Suspense>
      <WorkflowEditorScreen mode="new" />
    </Suspense>
  );
}
