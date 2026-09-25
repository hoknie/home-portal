import { Suspense } from "react";

import { ServicesScreen } from "@/widgets/services-table";

export default function ServicesPage() {
  return (
    <Suspense>
      <ServicesScreen />
    </Suspense>
  );
}
