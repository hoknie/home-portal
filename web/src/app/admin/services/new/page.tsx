import { Suspense } from "react";

import { ServiceEditorScreen } from "@/widgets/service-editor";

export default function NewServicePage() {
  return (
    <Suspense>
      <ServiceEditorScreen mode="new" />
    </Suspense>
  );
}
