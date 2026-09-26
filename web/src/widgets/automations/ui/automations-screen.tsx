"use client";

import { Plus, Workflow } from "lucide-react";
import Link from "next/link";
import { useTranslations } from "next-intl";
import { useState } from "react";

import { StopRunButton } from "@/features/stop-run";
import { RunDetails, useAutomations, useCatalogue } from "@/entities/automation";
import { routes } from "@/shared/config";
import { DataTable } from "@/shared/ui/data-table";
import { EmptyState } from "@/shared/ui/empty-state";
import { ErrorNotice } from "@/shared/ui/error-notice";
import { PageHeader } from "@/shared/ui/page-header";
import { Button, Skeleton } from "@/shared/ui/primitives";
import { SectionCard } from "@/shared/ui/section-card";
import { TagFilter, distinctTags, stillChosen, tagsMatch } from "@/shared/ui/tag-filter";

import { useAutomationColumns } from "./automation-columns";
import { RunJournal } from "./run-journal";

export function AutomationsScreen() {
  const t = useTranslations();
  const automations = useAutomations();
  const catalogue = useCatalogue();
  const [opened, setOpened] = useState<string | null>(null);
  const columns = useAutomationColumns(automations.data?.revision ?? null, catalogue.data, setOpened);
  const add = (
    <Button asChild>
      <Link href={routes.newAutomation}>
        <Plus aria-hidden />
        {t("automations.add")}
      </Link>
    </Button>
  );
  const [picked, setChosen] = useState<string[]>([]);
  const list = automations.data?.data.automations ?? [];
  const tags = distinctTags(list.map((automation) => automation.tags));
  const chosen = stillChosen(picked, tags);
  const shown = list.filter((automation) => tagsMatch(automation.tags, chosen));
  return (
    <div className="grid gap-8">
      <PageHeader title={t("automations.title")} description={t("automations.subtitle")} actions={add} />
      {automations.error && !automations.data ? (
        <ErrorNotice title={t("errors.loadFailed")} description={automations.error.message} onRetry={() => void automations.refetch()} />
      ) : null}
      {automations.data ? (
        <SectionCard flush actions={<TagFilter label={t("tags.filter")} tags={tags} selected={chosen} onChange={setChosen} />}>
          <DataTable
            columns={columns}
            rows={shown}
            rowKey={(automation) => automation.id}
            empty={
              chosen.length > 0 ? (
                <EmptyState icon={Workflow} title={t("tags.noMatches")} description={t("tags.noMatchesHint")} />
              ) : (
                <EmptyState icon={Workflow} title={t("automations.empty")} description={t("automations.emptyHint")} action={add} />
              )
            }
          />
        </SectionCard>
      ) : automations.error ? null : (
        <Skeleton className="h-64 w-full" aria-busy="true" />
      )}
      {automations.data ? <RunJournal automations={list} onOpen={setOpened} /> : null}
      <RunDetails
        runId={opened}
        onClose={() => setOpened(null)}
        actions={(run) => <StopRunButton run={run} title={list.find((automation) => automation.id === run.automation)?.title} labelled />}
      />
    </div>
  );
}
