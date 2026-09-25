import { Suspense } from "react";

import { AutomationEditorScreen } from "@/widgets/automation-editor";

export default function NewAutomationPage() {
  return (
    <Suspense>
      <AutomationEditorScreen mode="new" />
    </Suspense>
  );
}
