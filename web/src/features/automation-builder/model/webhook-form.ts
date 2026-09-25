import { z } from "zod";

import type { Catalogue, CatalogueEvent } from "@/entities/automation";
import type { Webhook, WebhookRequest } from "@/entities/webhook";

import { DEFAULT_TIMEOUT, LONGEST_TIMEOUT, scriptAccepted } from "./form";
import { VARIABLE_PREFIX, WEBHOOK_EVENT, eventOf } from "./events";
import { unknownPlaceholders } from "./placeholders";

export const VARIABLE_PATTERN = /^[a-z][a-z0-9_]{0,62}$/;
export const RESERVED_VARIABLES = ["id", "title"];

export const webhookFormSchema = (fieldsOf: (variables: string[]) => readonly string[]) =>
  z
    .object({
      title: z.string().trim().min(1, "validation.automationTitle").max(120, "validation.automationTitle"),
      enabled: z.boolean(),
      tags: z.array(z.string().trim().min(1, "validation.tags").max(40, "validation.tags")).max(20, "validation.tags"),
      variables: z
        .array(z.string())
        .refine((names) => names.every((name) => VARIABLE_PATTERN.test(name) && !RESERVED_VARIABLES.includes(name)), "validation.webhookVariable"),
      action: z.enum(["event", "script"]),
      script: z.string().trim(),
      args: z.array(z.object({ value: z.string() })),
      timeout_seconds: z.number({ error: "validation.automationTimeout" }).int().min(1, "validation.automationTimeout").max(LONGEST_TIMEOUT, "validation.automationTimeout"),
      with_token: z.boolean(),
    })
    .superRefine((form, context) => {
      if (form.action !== "script") {
        return;
      }
      if (!scriptAccepted(form.script)) {
        context.addIssue({ code: "custom", path: ["script"], message: "validation.automationScript" });
      }
      const allowed = fieldsOf(form.variables);
      form.args.forEach((argument, index) => {
        if (unknownPlaceholders(argument.value, allowed).length > 0) {
          context.addIssue({ code: "custom", path: ["args", index, "value"], message: "validation.automationPlaceholder" });
        }
      });
    });

export type WebhookForm = z.infer<ReturnType<typeof webhookFormSchema>>;

export const emptyWebhookForm: WebhookForm = {
  title: "",
  enabled: true,
  tags: [],
  variables: [],
  action: "script",
  script: "",
  args: [],
  timeout_seconds: DEFAULT_TIMEOUT,
  with_token: true,
};

export function webhookFormOf(webhook: Webhook): WebhookForm {
  return {
    title: webhook.title,
    enabled: webhook.enabled,
    tags: [...webhook.tags],
    variables: [...webhook.variables],
    action: webhook.action,
    script: webhook.run?.script ?? "",
    args: (webhook.run?.args ?? []).map((value) => ({ value })),
    timeout_seconds: webhook.run?.timeout_seconds ?? DEFAULT_TIMEOUT,
    with_token: webhook.protected,
  };
}

export function webhookRequestOf(form: WebhookForm, creating: boolean): WebhookRequest {
  return {
    title: form.title.trim(),
    enabled: form.enabled,
    tags: form.tags,
    variables: form.variables,
    action: form.action,
    run:
      form.action === "script"
        ? { script: form.script.trim(), args: form.args.map((argument) => argument.value), timeout_seconds: form.timeout_seconds }
        : null,
    ...(creating ? { with_token: form.with_token } : {}),
  };
}

export function webhookEventOf(catalogue: Catalogue, variables: string[]): CatalogueEvent {
  const event = eventOf(catalogue, WEBHOOK_EVENT);
  return {
    ...event,
    fields: [...event.fields, ...variables.map((name) => ({ name: `${VARIABLE_PREFIX}${name}`, sample: name }))],
  };
}
