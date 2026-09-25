import { Suspense } from "react";

import { AutomationEditorScreen } from "@/widgets/automation-editor";

export default function EditAutomationPage() {
  return (
    <Suspense>
      <AutomationEditorScreen mode="edit" />
    </Suspense>
  );
}
