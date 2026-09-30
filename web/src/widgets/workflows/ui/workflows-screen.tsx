"use client";

import { Plus, Route } from "lucide-react";
import { useTranslations } from "next-intl";
import { useState } from "react";

import { Allowed } from "@/entities/session";
import { enabledModules, useModules } from "@/entities/module";
import { TemplatesGallery, useWorkflows } from "@/entities/workflow";
import { DataTable } from "@/shared/ui/data-table";
import { EmptyState } from "@/shared/ui/empty-state";
import { ErrorNotice } from "@/shared/ui/error-notice";
import { ModuleOffNotice } from "@/shared/ui/module-off-notice";
import { useTrail } from "@/shared/lib/breadcrumbs";
import { PageHeader } from "@/shared/ui/page-header";
import { Button, Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, Skeleton } from "@/shared/ui/primitives";
import { SectionCard } from "@/shared/ui/section-card";
import { TagFilter, distinctTags, stillChosen, tagsMatch } from "@/shared/ui/tag-filter";

import { useWorkflowColumns } from "./workflow-columns";

export function WorkflowsScreen() {
  const trail = useTrail();
  const t = useTranslations();
  const loadedModules = useModules().data?.data;
  const moduleOff = loadedModules !== undefined && !enabledModules(loadedModules).has("workflows");
  const workflows = useWorkflows();
  const columns = useWorkflowColumns({ revision: workflows.data?.revision ?? null, moduleOff });
  const [picked, setChosen] = useState<string[]>([]);
  const list = workflows.data?.data.workflows ?? [];
  const tags = distinctTags(list.map((workflow) => workflow.tags));
  const chosen = stillChosen(picked, tags);
  const [gallery, setGallery] = useState(false);
  const add = (
    <Allowed area="workflows" action="create">
      <Button type="button" onClick={() => setGallery(true)}>
        <Plus aria-hidden />
        {t("workflows.add")}
      </Button>
    </Allowed>
  );
  return (
    <div className="grid gap-8">
      <PageHeader breadcrumbs={trail.of(trail.section("workflows"))} title={t("workflows.title")} description={t("workflows.subtitle")} actions={add} />
      {moduleOff ? <ModuleOffNotice name={t("modules.names.workflows")} /> : null}
      {workflows.error && !workflows.data ? (
        <ErrorNotice title={t("errors.loadFailed")} description={workflows.error.message} onRetry={() => void workflows.refetch()} />
      ) : null}
      {workflows.data ? (
        <SectionCard flush actions={<TagFilter label={t("tags.filter")} tags={tags} selected={chosen} onChange={setChosen} />}>
          <DataTable
            columns={columns}
            rows={list.filter((workflow) => tagsMatch(workflow.tags, chosen))}
            rowKey={(workflow) => workflow.id}
            empty={
              chosen.length > 0 ? (
                <EmptyState icon={Route} title={t("tags.noMatches")} description={t("tags.noMatchesHint")} />
              ) : (
                <div className="grid gap-4 p-4">
                  <EmptyState icon={Route} title={t("workflows.empty")} description={t("workflows.emptyHint")} />
                  <p className="mx-auto max-w-2xl text-center text-sm text-muted-foreground">{t("workflows.explained")}</p>
                  <TemplatesGallery />
                </div>
              )
            }
          />
        </SectionCard>
      ) : workflows.error ? null : (
        <Skeleton className="h-64 w-full" aria-busy="true" />
      )}
      <Dialog open={gallery} onOpenChange={setGallery}>
        <DialogContent closeLabel={t("common.close")} className="max-h-[85dvh] overflow-y-auto sm:max-w-3xl">
          <DialogHeader>
            <DialogTitle>{t("workflows.chooseStart")}</DialogTitle>
            <DialogDescription>{t("workflows.chooseStartHint")}</DialogDescription>
          </DialogHeader>
          <TemplatesGallery />
        </DialogContent>
      </Dialog>
    </div>
  );
}
