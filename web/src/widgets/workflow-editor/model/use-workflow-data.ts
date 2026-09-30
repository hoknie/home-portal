"use client";

import { useMemo } from "react";

import { type Automation, type Catalogue, useAutomations, useCatalogue, useScripts } from "@/entities/automation";
import { useNotifications } from "@/entities/notification";
import { type EventKnowledge, type Workflow, usePortalValues, useSecretNames, useWorkflowCatalogue, useWorkflows } from "@/entities/workflow";

import type { Sources } from "./editor-context";

export const WEBHOOK_EVENT = "webhook.received";
export const MANUAL_EVENT = "manual";

const EVENT_PREFIX = /^event\./;
const WEBHOOK_PREFIX = "webhook.";

function startingEvents(workflow: Workflow | null, automations: Automation[]) {
  const events = new Set<string>();
  const variables = new Set<string>();
  let fromWebhook = false;
  for (const usage of workflow?.used_by ?? []) {
    const event = usage.kind === "webhook" ? WEBHOOK_EVENT : usage.kind === "automation" ? automations.find((candidate) => candidate.id === usage.id)?.when.event : undefined;
    if (event !== undefined) {
      events.add(event);
    }
    if (event === WEBHOOK_EVENT) {
      fromWebhook = true;
      usage.variables.forEach((name) => variables.add(name));
    }
  }
  if (events.size === 0) {
    events.add(MANUAL_EVENT);
  }
  return { events, variables: fromWebhook ? [...variables] : null };
}

export function eventFieldsFor(workflow: Workflow | null, automations: Automation[], catalogue: Catalogue | undefined) {
  const { events, variables } = startingEvents(workflow, automations);
  const fields = new Map<string, string>();
  for (const event of catalogue?.events ?? []) {
    if (events.has(event.name)) {
      event.fields.forEach((field) => fields.set(field.name.replace(EVENT_PREFIX, ""), field.sample));
    }
  }
  (variables ?? []).forEach((name) => fields.set(`${WEBHOOK_PREFIX}${name}`, name));
  return [...fields].map(([name, sample]) => ({ name, sample }));
}

export function eventKnowledgeFor(workflow: Workflow | null, automations: Automation[], catalogue: Catalogue | undefined): EventKnowledge {
  return {
    fields: [...new Set((catalogue?.events ?? []).flatMap((event) => event.fields.map((field) => field.name)))],
    variables: startingEvents(workflow, automations).variables,
  };
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
  const loadedAutomations = automations.data?.data.automations;
  const eventKnowledge = useMemo(() => eventKnowledgeFor(workflow, loadedAutomations ?? [], events.data), [workflow, loadedAutomations, events.data]);
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
    eventKnowledge,
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
