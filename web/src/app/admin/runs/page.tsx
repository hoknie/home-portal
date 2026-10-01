import { Suspense } from "react";

import { RunJournalScreen } from "@/widgets/automations";

export default function RunsPage() {
  return (
    <Suspense>
      <RunJournalScreen />
    </Suspense>
  );
}
