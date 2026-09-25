import { Suspense } from "react";

import { WebhookEditorScreen } from "@/widgets/webhook-editor";

export default function NewWebhookPage() {
  return (
    <Suspense>
      <WebhookEditorScreen mode="new" />
    </Suspense>
  );
}
