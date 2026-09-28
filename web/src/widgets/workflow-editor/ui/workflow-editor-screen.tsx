"use client";

import { SearchX } from "lucide-react";
import Link from "next/link";
import { useRouter, useSearchParams } from "next/navigation";
import { useTranslations } from "next-intl";

import { type Automation, type Catalogue, useAutomations, useCatalogue, useScripts } from "@/entities/automation";
import { useNotifications } from "@/entities/notification";
import { TEMPLATE_PARAMETER, type Workflow, usePortalValues, useSecretNames, useWorkflowCatalogue, useWorkflows } from "@/entities/workflow";
import { routes } from "@/shared/config";
import { slugOf, uniqueId } from "@/shared/lib/slug";
import { EmptyState } from "@/shared/ui/empty-state";
import { ErrorNotice } from "@/shared/ui/error-notice";
import { useTrail } from "@/shared/lib/breadcrumbs";
import { PageHeader } from "@/shared/ui/page-header";
import { Button, Skeleton } from "@/shared/ui/primitives";

import { localizedTemplate } from "../model/localize";
import { WorkflowEditor } from "./workflow-editor";

export const ID_PARAMETER = "id";
export const WEBHOOK_EVENT = "webhook.received";
export const MANUAL_EVENT = "manual";

export function eventFieldsFor(workflow: Workflow | null, automations: Automation[], catalogue: Catalogue | undefined) {
  const events = new Set<string>();
  for (const usage of workflow?.used_by ?? []) {
    if (usage.kind === "automation") {
      const automation = automations.find((candidate) => candidate.id === usage.id);
      if (automation) {
        events.add(automation.when.event);
      }
    } else if (usage.kind === "webhook") {
      events.add(WEBHOOK_EVENT);
    }
  }
  if (events.size === 0) {
    events.add(MANUAL_EVENT);
  }
  const fields = new Map<string, string>();
  for (const event of catalogue?.events ?? []) {
    if (events.has(event.name)) {
      event.fields.forEach((field) => fields.set(field.name, field.sample));
    }
  }
  return [...fields].map(([name, sample]) => ({ name, sample }));
}

export function WorkflowEditorScreen({ mode }: { mode: "new" | "edit" }) {
  const t = useTranslations();
  const help = useTranslations("workflowHelp");
  const trail = useTrail();
  const router = useRouter();
  const parameters = useSearchParams();
  const id = parameters.get(ID_PARAMETER) ?? "";
  const templateName = mode === "new" ? parameters.get(TEMPLATE_PARAMETER) : null;
  const workflows = useWorkflows();
  const catalogue = useWorkflowCatalogue();
  const scripts = useScripts();
  const automations = useAutomations();
  const events = useCatalogue();
  const secrets = useSecretNames();
  const notifications = useNotifications();
  const portal = usePortalValues();
  const all = workflows.data?.data.workflows ?? [];
  const workflow = mode === "edit" ? (all.find((candidate) => candidate.id === id) ?? null) : null;
  const last = mode === "new" ? t("breadcrumbs.new.workflow") : (workflow?.title ?? id);
  const header = <PageHeader breadcrumbs={trail.of(trail.section("workflows"), { label: last })} title={t(mode === "new" ? "workflowEditor.addTitle" : "workflowEditor.editTitle")} description={t("workflowEditor.pageDescription")} />;
  const failure = workflows.error ?? catalogue.error;
  if (!workflows.data || !catalogue.data) {
    return (
      <div className="grid gap-6">
        {header}
        {failure ? (
          <ErrorNotice
            title={t("errors.loadFailed")}
            description={failure.message}
            onRetry={() => {
              void workflows.refetch();
              void catalogue.refetch();
            }}
          />
        ) : (
          <Skeleton className="h-96 w-full" aria-busy="true" />
        )}
      </div>
    );
  }
  if (mode === "edit" && !workflow) {
    return (
      <div className="grid gap-6">
        {header}
        <EmptyState
          icon={SearchX}
          title={t("workflowEditor.notFound")}
          description={t("workflowEditor.notFoundHint")}
          action={
            <Button asChild variant="outline">
              <Link href={routes.adminWorkflows}>{t("workflowEditor.backToWorkflows")}</Link>
            </Button>
          }
        />
      </div>
    );
  }
  const taken = all.map((candidate) => candidate.id);
  const template = localizedTemplate(templateName, { has: (key) => help.has(key as "kinds.if.name"), text: (key) => help(key as "kinds.if.name") });
  const initial = template ? { ...template, id: template.title ? uniqueId(slugOf(template.title), taken) : "" } : null;
  return (
    <div className="grid gap-4">
      {header}
      <WorkflowEditor
        key={workflow?.id ?? `new-${templateName ?? ""}`}
        workflow={workflow}
        initial={initial}
        revision={workflows.data.revision}
        workflows={all}
        catalogue={catalogue.data}
        sources={{
          services: events.data?.choices.services ?? [],
          states: events.data?.states ?? [],
          scripts: scripts.data?.scripts ?? [],
          secrets: secrets.data ?? [],
          channels: notifications.data?.data.channels ?? [],
          portal: portal.data ?? null,
          automations: (automations.data?.data.automations ?? []).map((automation) => ({
            id: automation.id,
            title: automation.title,
            event: automation.when.event,
            enabled: automation.enabled,
          })),
          eventFields: eventFieldsFor(workflow, automations.data?.data.automations ?? [], events.data),
        }}
        tags={[...new Set([...(events.data?.choices.tags ?? []), ...all.flatMap((candidate) => candidate.tags)])]}
        onSaved={() => router.push(routes.adminWorkflows)}
        onConflict={() => void workflows.refetch()}
      />
    </div>
  );
}
