import { Suspense } from "react";

import { LayoutEditorScreen } from "@/widgets/layout-editor";

export default function LayoutPage() {
  return (
    <Suspense>
      <LayoutEditorScreen />
    </Suspense>
  );
}
