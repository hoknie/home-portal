"use client";

import { type Automation, type Catalogue, useAutomations, useCatalogue, useScripts } from "@/entities/automation";
import { useNotifications } from "@/entities/notification";
import { type Workflow, usePortalValues, useSecretNames, useWorkflowCatalogue, useWorkflows } from "@/entities/workflow";

import type { Sources } from "./editor-context";

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

export function useWorkflowData(id: string | null) {
  const workflows = useWorkflows();
  const catalogue = useWorkflowCatalogue();
  const scripts = useScripts();
  const automations = useAutomations();
  const events = useCatalogue();
  const secrets = useSecretNames();
  const notifications = useNotifications();
  const portal = usePortalValues();
  const all = workflows.data?.data.workflows ?? [];
  const workflow = id === null ? null : (all.find((candidate) => candidate.id === id) ?? null);
  const sources: Omit<Sources, "workflows"> = {
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
  };
  return {
    ready: workflows.data !== undefined && catalogue.data !== undefined,
    failure: workflows.error ?? catalogue.error,
    retry: () => {
      void workflows.refetch();
      void catalogue.refetch();
    },
    refetch: () => void workflows.refetch(),
    all,
    workflow,
    revision: workflows.data?.revision ?? null,
    catalogue: catalogue.data,
    sources,
    tags: [...new Set([...(events.data?.choices.tags ?? []), ...all.flatMap((candidate) => candidate.tags)])],
  };
}
