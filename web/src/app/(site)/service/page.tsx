import { Suspense } from "react";

import { ServicePageScreen } from "@/widgets/service-page";

export default function ServicePage() {
  return (
    <Suspense>
      <ServicePageScreen />
    </Suspense>
  );
}
