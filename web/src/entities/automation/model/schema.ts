import { z } from "zod";

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

export const OUTCOMES = ["succeeded", "failed", "timed-out", "skipped", "refused"] as const;

export const outcomeSchema = z.enum([...OUTCOMES, UNKNOWN]).catch(UNKNOWN);

export type Outcome = z.infer<typeof outcomeSchema>;

export const FILTER_NAMES = ["services", "from", "to", "from_unknown", "users", "environments", "webhooks", "cron"] as const;

export type FilterName = (typeof FILTER_NAMES)[number];

const outputSchema = z.object({ tail: z.string(), bytes: z.number(), truncated: z.boolean() });

export const runSchema = z.object({
  id: z.string(),
  automation: z.string(),
  event: eventNameSchema,
  fields: z.record(z.string(), z.string()),
  arguments: z.array(z.string()),
  started_at: z.string(),
  outcome: z.object({
    result: outcomeSchema,
    exit_code: z.number().nullable(),
    reason: z.string().nullable(),
    duration_milliseconds: z.number(),
    count: z.number().default(1),
    last_at: z.string(),
    stdout: outputSchema,
    stderr: outputSchema,
  }),
});

export type Run = z.infer<typeof runSchema>;

export const runsSchema = z.object({ runs: z.array(runSchema) });

export const whenSchema = z.object({
  event: eventNameSchema,
  cron: z.string().optional(),
  services: z.array(z.string()).default([]),
  from: z.array(z.string()).default([]),
  to: z.array(z.string()).default([]),
  from_unknown: z.boolean().default(false),
  users: z.array(z.string()).default([]),
  environments: z.array(z.string()).default([]),
  webhooks: z.array(z.string()).default([]),
});

export type When = z.infer<typeof whenSchema>;

export const automationSchema = z.object({
  id: z.string(),
  title: z.string(),
  enabled: z.boolean(),
  tags: z.array(z.string()).default([]),
  cooldown_seconds: z.number(),
  when: whenSchema,
  run: z.object({ script: z.string(), args: z.array(z.string()), timeout_seconds: z.number() }),
  last_run: runSchema.nullable().default(null),
});

export type Automation = z.infer<typeof automationSchema>;

export const automationsSchema = z.object({ automations: z.array(automationSchema) });

export const catalogueEventSchema = z.object({
  name: z.string(),
  fields: z.array(z.object({ name: z.string(), sample: z.string() })),
  filters: z.array(z.string()),
});

export type CatalogueEvent = z.infer<typeof catalogueEventSchema>;

export const catalogueSchema = z.object({
  events: z.array(catalogueEventSchema),
  states: z.array(z.string()),
  choices: z.object({
    services: z.array(z.object({ id: z.string(), name: z.string() })),
    users: z.array(z.string()),
    environments: z.array(z.string()),
    webhooks: z.array(z.object({ id: z.string(), name: z.string(), variables: z.array(z.string()) })).default([]),
    tags: z.array(z.string()).default([]),
  }),
});

export type Catalogue = z.infer<typeof catalogueSchema>;

export const scriptsSchema = z.object({
  directory: z.string(),
  exists: z.boolean(),
  user_id: z.number().default(0),
  scripts: z.array(
    z.object({
      path: z.string(),
      runnable: z.boolean(),
      problem: z.string().nullable(),
      code: z.string().nullable().default(null),
      concerns: z.string().nullable().default(null),
    }),
  ),
});

export type Scripts = z.infer<typeof scriptsSchema>;

export const scheduleSchema = z.object({ timezone: z.string(), times: z.array(z.string()) });

export type Schedule = z.infer<typeof scheduleSchema>;

export const queuedSchema = z.object({ run_id: z.string() });

export function messageKeyOf(name: string) {
  return name.replace(/[.-]/g, "_");
}
