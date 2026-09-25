import { Suspense } from "react";

import { WebhookEditorScreen } from "@/widgets/webhook-editor";

export default function EditWebhookPage() {
  return (
    <Suspense>
      <WebhookEditorScreen mode="edit" />
    </Suspense>
  );
}
