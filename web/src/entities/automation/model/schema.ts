import { z } from "zod";

import { generated } from "@/shared/api";

import { scriptArgumentSchema } from "@/entities/script/@x/automation";

export const EVENT_NAMES = [
  "schedule",
  "portal.started",
  "portal.stopping",
  "service.status-changed",
  "service.created",
  "service.updated",
  "service.deleted",
  "user.signed-in",
  "user.signed-out",
  "user.sign-in-failed",
  "configuration.changed",
  "webhook.received",
  "manual",
] as const;

export const UNKNOWN = "unknown";

export const eventNameSchema = z.enum([...EVENT_NAMES, UNKNOWN]).catch(UNKNOWN);

export type EventName = z.infer<typeof eventNameSchema>;

export const OUTCOMES = ["queued", "running", "succeeded", "failed", "timed-out", "stopped", "skipped", "refused"] as const;

export const ACTIVE_OUTCOMES: readonly Outcome[] = ["queued", "running"];

export const outcomeSchema = z.enum([...OUTCOMES, UNKNOWN]).catch(UNKNOWN);

export type Outcome = z.infer<typeof outcomeSchema>;

export const FILTER_NAMES = ["services", "from", "to", "from_unknown", "users", "environments", "webhooks", "cron"] as const;

export type FilterName = (typeof FILTER_NAMES)[number];

export const STEP_OUTCOMES = ["running", "succeeded", "failed", "skipped", "stopped", "timed-out"] as const;

export const stepOutcomeSchema = z.enum([...STEP_OUTCOMES, UNKNOWN]).catch(UNKNOWN);

export type StepOutcome = z.infer<typeof stepOutcomeSchema>;

export const LOG_LEVELS = ["info", "warning", "error"] as const;

export type LogLevel = (typeof LOG_LEVELS)[number];

const served = generated.automations;

export const traceEntrySchema = served.traceEntryResponseSchema.extend({ outcome: stepOutcomeSchema, level: z.enum(LOG_LEVELS).nullable().catch(null).default(null) });

export type TraceEntry = z.infer<typeof traceEntrySchema>;

export const traceSchema = served.traceResponseSchema.extend({ entries: z.array(traceEntrySchema) });

export type Trace = z.infer<typeof traceSchema>;

export const runSchema = served.runResponseSchema.extend({
  event: eventNameSchema,
  outcome: served.outcomeResponseSchema.extend({ result: outcomeSchema }),
  trace: traceSchema.nullable().default(null),
});

export type Run = z.infer<typeof runSchema>;

export function isActive(run: Run | null | undefined) {
  return run ? ACTIVE_OUTCOMES.includes(run.outcome.result) : false;
}

export const runsSchema = z.object({ runs: z.array(runSchema) });

export const whenSchema = served.whenResponseSchema.extend({ event: eventNameSchema });

export type When = z.infer<typeof whenSchema>;

export const automationSchema = served.automationResponseSchema.extend({
  when: whenSchema,
  last_run: runSchema.nullable().default(null),
  active_run: runSchema.nullable().default(null),
});

export type Automation = z.infer<typeof automationSchema>;

export const automationsSchema = z.object({ automations: z.array(automationSchema) });

export const catalogueEventSchema = generated.automationCatalogue.eventResponseSchema;

export type CatalogueEvent = z.infer<typeof catalogueEventSchema>;

const choices = generated.automationCatalogue.choicesResponseSchema;

export const catalogueSchema = generated.automationCatalogue.catalogueResponseSchema.extend({
  choices: choices.extend({ webhooks: z.array(generated.automationCatalogue.webhookChoiceResponseSchema.extend({ action: z.enum(["event", "script", "workflow"]).catch("event") })) }),
});

export type Catalogue = z.infer<typeof catalogueSchema>;

export const scriptsSchema = generated.automationScripts.scriptsResponseSchema.extend({
  scripts: z.array(generated.automationScripts.scriptResponseSchema.extend({ arguments: z.array(scriptArgumentSchema) })),
});

export type Scripts = z.infer<typeof scriptsSchema>;

export const scheduleSchema = generated.automationSchedule.scheduleResponseSchema;

export type Schedule = z.infer<typeof scheduleSchema>;

export const queuedSchema = generated.automationQueued.queuedResponseSchema;

export function messageKeyOf(name: string) {
  return name.replace(/[.-]/g, "_");
}
