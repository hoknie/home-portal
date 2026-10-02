import { Suspense } from "react";

import { WidgetLibraryScreen } from "@/widgets/layout-editor";

export default function WidgetLibraryPage() {
  return (
    <Suspense>
      <WidgetLibraryScreen />
    </Suspense>
  );
}
