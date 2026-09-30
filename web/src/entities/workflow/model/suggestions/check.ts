import type { Scope } from "../scope";
import { type FilterCall } from "../transforming/parse";
import { NAMESPACES, placeholderAt } from "../transforming/render";
import { type PortalValues, portalProblem } from "../portal";
import { certainType, chainProblem } from "../transforming/types";

export { NAMESPACES };

export const REASONS = [
  "unknownInput",
  "unknownVariable",
  "unknownStep",
  "outsideLoop",
  "outsideTransform",
  "notAValue",
  "notInPortal",
  "unreadableFilter",
  "unknownFilter",
  "filterArguments",
  "filterArgument",
  "filterType",
  "unknownEventField",
  "undeclaredWebhookVariable",
] as const;

export const WARNING_REASONS: readonly Reason[] = ["undeclaredWebhookVariable"];

export type EventKnowledge = { fields: string[]; variables: string[] | null };

const WEBHOOK = "webhook.";
const BODY = "webhook.body";
const VARIABLE = /^[a-z][a-z0-9_]*$/;

function eventReason(field: string, events: EventKnowledge): { reason: Reason; params: Record<string, string> } | null {
  if (field === "name" || field === "at" || field === BODY || field.startsWith(`${BODY}.`) || events.fields.includes(field)) {
    return null;
  }
  const variable = field.startsWith(WEBHOOK) ? field.slice(WEBHOOK.length) : null;
  if (variable !== null && VARIABLE.test(variable)) {
    return events.variables === null || events.variables.includes(variable) ? null : { reason: "undeclaredWebhookVariable", params: { name: variable } };
  }
  return { reason: "unknownEventField", params: { name: `event.${field}` } };
}

export type Reason = (typeof REASONS)[number];

export type TemplateName = { start: number; end: number; name: string; valid: boolean; filters: FilterCall[] | null; filterError: string | null };

export type TemplateProblem = { start: number; end: number; name: string; reason: Reason; params: Record<string, string>; warning: boolean };

export function templateNames(text: string): TemplateName[] {
  const found: TemplateName[] = [];
  let from = 0;
  for (;;) {
    const open = text.indexOf("{{", from);
    if (open < 0) {
      return found;
    }
    const close = text.indexOf("}}", open + 2);
    if (close < 0) {
      return found;
    }
    const placeholder = placeholderAt(text.slice(open + 2));
    if (placeholder === null) {
      found.push({ start: open, end: close + 2, name: text.slice(open + 2, close), valid: false, filters: [], filterError: null });
      from = open + 1;
    } else {
      const end = open + 2 + placeholder.length + 2;
      found.push({ start: open, end, name: placeholder.name, valid: true, filters: placeholder.filters, filterError: placeholder.filterError });
      from = end;
    }
  }
}

function reasonFor(name: string, scope: Scope, portal: PortalValues | null, events: EventKnowledge | null): { reason: Reason; params: Record<string, string> } | null {
  const [namespace, first = "", second = ""] = name.split(".");
  switch (namespace) {
    case "event":
      return events === null ? null : eventReason(name.slice("event.".length), events);
    case "secrets":
      return null;
    case "inputs":
      return scope.inputs.includes(first) ? null : { reason: "unknownInput", params: { name: first } };
    case "vars":
      return scope.vars.includes(first) ? null : { reason: "unknownVariable", params: { name: first } };
    case "steps":
      return scope.steps.some((step) => step.id === first) ? null : { reason: "unknownStep", params: { name: first } };
    case "loop":
      return scope.inLoop && (first === "item" || first === "index") ? null : { reason: "outsideLoop", params: { name: `${first}${second ? `.${second}` : ""}` } };
    case "portal":
      return portalProblem(name, portal) ? { reason: "notInPortal", params: { name } } : null;
    case "item":
    case "index":
      return scope.inTransform ? null : { reason: "outsideTransform", params: { name } };
    default:
      return { reason: "notAValue", params: { name } };
  }
}

function filterReason(found: TemplateName, scope: Scope): { reason: Reason; params: Record<string, string> } | null {
  if (found.filters === null) {
    return { reason: "unreadableFilter", params: { filter: found.filterError ?? "" } };
  }
  return chainProblem(certainType(found.name, scope), found.filters);
}

export function checkTemplate(text: string, scope: Scope, portal: PortalValues | null = null, events: EventKnowledge | null = null): TemplateProblem[] {
  return templateNames(text).flatMap((found) => {
    const argumentProblem = (found.filters ?? []).flatMap((call) => call.names ?? []).map((named) => reasonFor(named.name, scope, portal, events)).find((reason) => reason !== null) ?? null;
    const problem = found.valid
      ? (argumentProblem ?? reasonFor(found.name, scope, portal, events) ?? filterReason(found, scope))
      : found.name.includes(".")
        ? { reason: "notAValue" as const, params: { name: found.name } }
        : null;
    return problem ? [{ start: found.start, end: found.end, name: found.name, ...problem, warning: WARNING_REASONS.includes(problem.reason) }] : [];
  });
}
