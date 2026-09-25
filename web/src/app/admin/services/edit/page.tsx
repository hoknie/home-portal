import { Suspense } from "react";

import { ServiceEditorScreen } from "@/widgets/service-editor";

export default function EditServicePage() {
  return (
    <Suspense>
      <ServiceEditorScreen mode="edit" />
    </Suspense>
  );
}
