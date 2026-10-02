import { Suspense } from "react";

import { WidgetBuilderScreen } from "@/widgets/layout-editor";

export default function WidgetBuilderPage() {
  return (
    <Suspense>
      <WidgetBuilderScreen mode="edit" />
    </Suspense>
  );
}
