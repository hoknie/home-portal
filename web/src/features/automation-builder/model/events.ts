import type { Catalogue, CatalogueEvent } from "@/entities/automation";

export const EVENT_GROUPS = ["manual", "schedule", "portal", "services", "users", "webhooks", "configuration"] as const;

export const WEBHOOK_EVENT = "webhook.received";
export const VARIABLE_PREFIX = "webhook.";

export type EventGroup = (typeof EVENT_GROUPS)[number];

export function groupOf(event: string): EventGroup {
  if (event === "schedule" || event === "manual") {
    return event;
  }
  if (event.startsWith("webhook.")) {
    return "webhooks";
  }
  if (event.startsWith("portal.")) {
    return "portal";
  }
  if (event.startsWith("service.")) {
    return "services";
  }
  if (event.startsWith("user.")) {
    return "users";
  }
  return "configuration";
}

export function eventOf(catalogue: Catalogue, name: string): CatalogueEvent {
  return catalogue.events.find((event) => event.name === name) ?? { name, fields: [], filters: [] };
}

export function withVariables(catalogue: Catalogue, name: string, chosen: string[]): CatalogueEvent {
  const event = eventOf(catalogue, name);
  if (name !== WEBHOOK_EVENT) {
    return event;
  }
  const webhooks = catalogue.choices.webhooks.filter((webhook) => chosen.length === 0 || chosen.includes(webhook.id));
  const variables = webhooks.length === 0 ? [] : webhooks[0].variables.filter((variable) => webhooks.every((webhook) => webhook.variables.includes(variable)));
  return { ...event, fields: [...event.fields, ...variables.map((variable) => ({ name: `${VARIABLE_PREFIX}${variable}`, sample: variable }))] };
}

export function fieldsOf(catalogue: Catalogue) {
  return (name: string, webhooks: string[]) => withVariables(catalogue, name, webhooks).fields.map((field) => field.name);
}

export function samplesOf(event: CatalogueEvent, chosen: Record<string, string | undefined>) {
  return Object.fromEntries(event.fields.map((field) => [field.name, chosen[field.name] ?? field.sample]));
}
