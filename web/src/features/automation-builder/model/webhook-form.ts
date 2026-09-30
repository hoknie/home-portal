import { z } from "zod";

import type { Catalogue, CatalogueEvent } from "@/entities/automation";
import type { Webhook, WebhookRequest } from "@/entities/webhook";

import { DEFAULT_TIMEOUT, LONGEST_TIMEOUT, scriptAccepted } from "./form";
import { VARIABLE_PREFIX, WEBHOOK_EVENT, eventOf } from "./events";
import type { InputDeclaration } from "@/entities/workflow";

import { unknownPlaceholders } from "./placeholders";
import { entriesOf, inputEntrySchema, inputPlaceholderIssues, inputsRequest } from "./workflow-call";

export const VARIABLE_PATTERN = /^[a-z][a-z0-9_]{0,62}$/;
export const RESERVED_VARIABLES = ["id", "title", "body"];
export const WEBHOOK_FORM_ACTIONS = ["script", "workflow", "event"] as const;

export const webhookFormSchema = (fieldsOf: (variables: string[]) => readonly string[]) =>
  z
    .object({
      title: z.string().trim().min(1, "validation.automationTitle").max(120, "validation.automationTitle"),
      enabled: z.boolean(),
      tags: z.array(z.string().trim().min(1, "validation.tags").max(40, "validation.tags")).max(20, "validation.tags"),
      variables: z
        .array(z.string())
        .refine((names) => names.every((name) => VARIABLE_PATTERN.test(name) && !RESERVED_VARIABLES.includes(name)), "validation.webhookVariable"),
      action: z.enum(WEBHOOK_FORM_ACTIONS),
      script: z.string().trim(),
      args: z.array(z.object({ value: z.string() })),
      timeout_seconds: z.number({ error: "validation.automationTimeout" }),
      workflow: z.string().trim(),
      inputs: z.record(z.string(), inputEntrySchema),
      with_token: z.boolean(),
    })
    .superRefine((form, context) => {
      if (form.action === "workflow") {
        if (form.workflow === "") {
          context.addIssue({ code: "custom", path: ["workflow"], message: "validation.automationWorkflow" });
        }
        for (const name of inputPlaceholderIssues(form.inputs, fieldsOf(form.variables))) {
          context.addIssue({ code: "custom", path: ["inputs", name], message: "validation.automationPlaceholder" });
        }
        return;
      }
      if (form.action !== "script") {
        return;
      }
      if (!Number.isInteger(form.timeout_seconds) || form.timeout_seconds < 1 || form.timeout_seconds > LONGEST_TIMEOUT) {
        context.addIssue({ code: "custom", path: ["timeout_seconds"], message: "validation.automationTimeout" });
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
  workflow: "",
  inputs: {},
  with_token: true,
};

export function webhookFormOf(webhook: Webhook): WebhookForm {
  return {
    title: webhook.title,
    enabled: webhook.enabled,
    tags: [...webhook.tags],
    variables: [...webhook.variables],
    action: webhook.workflow ? "workflow" : webhook.action,
    script: webhook.run?.script ?? "",
    args: (webhook.run?.args ?? []).map((value) => ({ value })),
    timeout_seconds: webhook.run?.timeout_seconds ?? DEFAULT_TIMEOUT,
    workflow: webhook.workflow?.id ?? "",
    inputs: entriesOf(webhook.workflow?.inputs),
    with_token: webhook.protected,
  };
}

export function webhookRequestOf(form: WebhookForm, creating: boolean, declarations?: readonly InputDeclaration[]): WebhookRequest {
  const common = { title: form.title.trim(), enabled: form.enabled, tags: form.tags, variables: form.variables, ...(creating ? { with_token: form.with_token } : {}) };
  if (form.action === "workflow") {
    return { ...common, action: "script", run: null, workflow: form.workflow, inputs: inputsRequest(form.inputs, declarations) };
  }
  return {
    ...common,
    action: form.action,
    run: form.action === "script" ? { script: form.script.trim(), args: form.args.map((argument) => argument.value), timeout_seconds: form.timeout_seconds } : null,
  };
}

export function webhookEventOf(catalogue: Catalogue, variables: string[]): CatalogueEvent {
  const event = eventOf(catalogue, WEBHOOK_EVENT);
  return {
    ...event,
    fields: [...event.fields, ...variables.map((name) => ({ name: `${VARIABLE_PREFIX}${name}`, sample: name }))],
  };
}
