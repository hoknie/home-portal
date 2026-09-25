import { Suspense } from "react";

import { WebhookDetailsScreen } from "@/widgets/webhook-details";

export default function WebhookDetailsPage() {
  return (
    <Suspense>
      <WebhookDetailsScreen />
    </Suspense>
  );
}
