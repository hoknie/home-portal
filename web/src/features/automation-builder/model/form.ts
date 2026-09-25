import { z } from "zod";

import { type Automation, type AutomationRequest, EVENT_NAMES, type FilterName } from "@/entities/automation";

import { unknownPlaceholders } from "./placeholders";

export const ID_PATTERN = /^[a-z0-9-]{1,63}$/;
export const RESERVED_IDS = ["runs", "catalogue", "scripts", "schedule"] as const;
export const LONGEST_TIMEOUT = 3600;
export const DEFAULT_TIMEOUT = 60;
export const UNKNOWN_STATE = "unknown";

const ABSOLUTE = /^\//;
const CLIMBS = /(^|\/)\.\.(\/|$)/;

export function scriptAccepted(script: string) {
  const trimmed = script.trim();
  return trimmed !== "" && !ABSOLUTE.test(trimmed) && !CLIMBS.test(trimmed);
}

const baseSchema = z.object({
  id: z
    .string()
    .trim()
    .regex(ID_PATTERN, "validation.automationId")
    .refine((id) => !(RESERVED_IDS as readonly string[]).includes(id), "validation.automationReserved"),
  title: z.string().trim().min(1, "validation.automationTitle").max(120, "validation.automationTitle"),
  enabled: z.boolean(),
  tags: z.array(z.string().trim().min(1, "validation.tags").max(40, "validation.tags")).max(20, "validation.tags"),
  cooldown_seconds: z.number({ error: "validation.automationCooldown" }).int("validation.automationCooldown").min(0, "validation.automationCooldown"),
  event: z.enum(EVENT_NAMES),
  cron: z.string().trim(),
  services: z.array(z.string()),
  from: z.array(z.string()),
  to: z.array(z.string()),
  from_unknown: z.boolean(),
  users: z.array(z.string()),
  environments: z.array(z.string()),
  webhooks: z.array(z.string()),
  script: z.string().trim().refine(scriptAccepted, "validation.automationScript"),
  args: z.array(z.object({ value: z.string() })),
  timeout_seconds: z
    .number({ error: "validation.automationTimeout" })
    .int("validation.automationTimeout")
    .min(1, "validation.automationTimeout")
    .max(LONGEST_TIMEOUT, "validation.automationTimeout"),
});

export type AutomationForm = z.infer<typeof baseSchema>;

export function automationFormSchema(fieldsOf: (event: string, webhooks: string[]) => readonly string[]) {
  return baseSchema.superRefine((form, context) => {
    if (form.event === "schedule" && form.cron === "") {
      context.addIssue({ code: "custom", path: ["cron"], message: "validation.automationCron" });
    }
    if (form.from.includes(UNKNOWN_STATE) && !form.from_unknown) {
      context.addIssue({ code: "custom", path: ["from"], message: "validation.automationFromUnknown" });
    }
    const allowed = fieldsOf(form.event, form.webhooks);
    form.args.forEach((argument, index) => {
      if (unknownPlaceholders(argument.value, allowed).length > 0) {
        context.addIssue({ code: "custom", path: ["args", index, "value"], message: "validation.automationPlaceholder" });
      }
    });
  });
}

export const emptyAutomationForm: AutomationForm = {
  id: "",
  title: "",
  enabled: true,
  tags: [],
  cooldown_seconds: 0,
  event: "service.status-changed",
  cron: "",
  services: [],
  from: [],
  to: [],
  from_unknown: false,
  users: [],
  environments: [],
  webhooks: [],
  script: "",
  args: [],
  timeout_seconds: DEFAULT_TIMEOUT,
};

export function formOf(automation: Automation): AutomationForm {
  const { when, run } = automation;
  return {
    id: automation.id,
    title: automation.title,
    enabled: automation.enabled,
    tags: [...automation.tags],
    cooldown_seconds: automation.cooldown_seconds,
    event: when.event === "unknown" ? emptyAutomationForm.event : when.event,
    cron: when.cron ?? "",
    services: [...when.services],
    from: [...when.from],
    to: [...when.to],
    from_unknown: when.from_unknown,
    users: [...when.users],
    environments: [...when.environments],
    webhooks: [...when.webhooks],
    script: run.script,
    args: run.args.map((value) => ({ value })),
    timeout_seconds: run.timeout_seconds,
  };
}

export function requestOf(form: AutomationForm, filters: readonly string[]): AutomationRequest {
  const when: Record<string, unknown> = { event: form.event };
  const lists = {
    services: form.services,
    from: form.from,
    to: form.to,
    users: form.users,
    environments: form.environments,
    webhooks: form.webhooks,
  };
  for (const [name, values] of Object.entries(lists)) {
    if (filters.includes(name) && values.length > 0) {
      when[name] = values;
    }
  }
  if (filters.includes("from_unknown") && form.from_unknown) {
    when.from_unknown = true;
  }
  if (filters.includes("cron")) {
    when.cron = form.cron.trim();
  }
  return {
    id: form.id.trim(),
    title: form.title.trim(),
    enabled: form.enabled,
    tags: form.tags,
    cooldown_seconds: form.cooldown_seconds,
    when,
    run: { script: form.script.trim(), args: form.args.map((argument) => argument.value), timeout_seconds: form.timeout_seconds },
  };
}

export const CLEARED: Record<FilterName, string[] | boolean | string> = {
  services: [],
  from: [],
  to: [],
  from_unknown: false,
  users: [],
  environments: [],
  webhooks: [],
  cron: "",
};

export function foreignFilters(filters: readonly string[]) {
  return (Object.keys(CLEARED) as FilterName[]).filter((name) => !filters.includes(name));
}
