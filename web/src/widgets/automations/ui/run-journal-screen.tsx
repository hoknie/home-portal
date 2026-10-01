"use client";

import { useRouter, useSearchParams } from "next/navigation";
import { useTranslations } from "next-intl";
import { useState } from "react";

import { RunDetails, useAutomations } from "@/entities/automation";
import { StopRunButton } from "@/features/runs/stop-run";
import { routes } from "@/shared/config";
import { useTrail } from "@/shared/lib/breadcrumbs";
import { PageHeader } from "@/shared/ui/page-header";

import { RunJournal } from "./run-journal";

export const RUN_PARAMETER = "run";

export function RunJournalScreen() {
  const t = useTranslations("runJournal");
  const trail = useTrail();
  const router = useRouter();
  const [opened, setOpened] = useState(useSearchParams().get(RUN_PARAMETER));
  const automations = useAutomations().data?.data.automations ?? [];
  const open = (id: string | null) => {
    setOpened(id);
    router.replace(id === null ? routes.adminRuns : routes.run(id), { scroll: false });
  };
  return (
    <div className="grid gap-8">
      <PageHeader breadcrumbs={trail.of(trail.section("runs"))} title={t("title")} description={t("subtitle")} />
      <RunJournal automations={automations} onOpen={open} />
      <RunDetails
        runId={opened}
        onClose={() => open(null)}
        actions={(run) => <StopRunButton run={run} title={automations.find((automation) => automation.id === run.automation)?.title} labelled />}
      />
    </div>
  );
}
