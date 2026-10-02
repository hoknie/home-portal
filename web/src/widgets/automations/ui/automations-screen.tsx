"use client";

import { Plus, Workflow } from "lucide-react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { useTranslations } from "next-intl";
import { useState } from "react";

import { Allowed } from "@/entities/session";
import { useAutomations, useCatalogue } from "@/entities/automation";
import { enabledModules, useModules } from "@/entities/module";
import { routes } from "@/shared/config";
import { DataTable } from "@/shared/ui/data-table";
import { Appear, Button, EmptyState, ErrorNotice, SkeletonTable } from "@/shared/ui/kit";
import { ModuleOffNotice } from "@/shared/ui/module-off-notice";
import { useTrail } from "@/shared/lib/breadcrumbs";
import { PageHeader } from "@/shared/ui/page-header";
import { SectionCard } from "@/shared/ui/section-card";
import { TagFilter, distinctTags, stillChosen, tagsMatch } from "@/shared/ui/tag-filter";

import { useAutomationColumns } from "./automation-columns";

export function AutomationsScreen() {
  const trail = useTrail();
  const t = useTranslations();
  const loadedModules = useModules().data?.data;
  const automations = useAutomations();
  const catalogue = useCatalogue();
  const router = useRouter();
  const columns = useAutomationColumns(automations.data?.revision ?? null, catalogue.data, (id) => router.push(routes.run(id)));
  const add = (
    <Allowed area="automations" action="create">
      <Button asChild>
        <Link href={routes.newAutomation}>
          <Plus aria-hidden />
          {t("automations.add")}
        </Link>
      </Button>
    </Allowed>
  );
  const [picked, setChosen] = useState<string[]>([]);
  const list = automations.data?.data.automations ?? [];
  const tags = distinctTags(list.map((automation) => automation.tags));
  const chosen = stillChosen(picked, tags);
  const shown = list.filter((automation) => tagsMatch(automation.tags, chosen));
  return (
    <div className="grid gap-8">
      <PageHeader breadcrumbs={trail.of(trail.section("automations"))} title={t("automations.title")} description={t("automations.subtitle")} actions={add} />
      {loadedModules && !enabledModules(loadedModules).has("automations") ? <ModuleOffNotice name={t("modules.names.automations")} /> : null}
      {automations.error && !automations.data ? (
        <ErrorNotice title={t("errors.loadFailed")} description={automations.error.message} onRetry={() => void automations.refetch()} />
      ) : null}
      {automations.data ? (
        <Appear>
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
        </Appear>
      ) : automations.error ? null : (
        <SkeletonTable columns={4} rows={6} />
      )}
    </div>
  );
}
