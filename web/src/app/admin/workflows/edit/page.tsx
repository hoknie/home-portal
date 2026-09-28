import { Suspense } from "react";

import { WorkflowEditorScreen } from "@/widgets/workflow-editor";

export default function EditWorkflowPage() {
  return (
    <Suspense>
      <WorkflowEditorScreen mode="edit" />
    </Suspense>
  );
}
