import { z } from "zod";

import { generated } from "@/shared/api";

export const WEBHOOK_ACTIONS = ["event", "script"] as const;

export const webhookSchema = generated.webhooks.webhookResponseSchema.extend({ action: z.enum(WEBHOOK_ACTIONS).catch("event") });

export type Webhook = z.infer<typeof webhookSchema>;

export const webhooksSchema = z.object({ webhooks: z.array(webhookSchema) });

export const createdWebhookSchema = generated.webhookCreated.createdWebhookResponseSchema.extend({ webhook: webhookSchema });

export const tokenSchema = generated.webhookToken.tokenResponseSchema;

export const acceptedSchema = generated.webhookAccepted.acceptedResponseSchema;

export function absoluteAddress(address: string, origin: string) {
  return `${origin.replace(/\/$/, "")}${address}`;
}

export const SHORT_ID_EDGE = 4;

export function shortAddress(address: string) {
  const slash = address.lastIndexOf("/");
  const id = address.slice(slash + 1);
  if (id.length <= SHORT_ID_EDGE * 3) {
    return address;
  }
  return `${address.slice(0, slash + 1)}${id.slice(0, SHORT_ID_EDGE * 2)}…${id.slice(-SHORT_ID_EDGE)}`;
}

export function curlExample(url: string, token: string | null, variables: string[]) {
  const body = JSON.stringify(Object.fromEntries(variables.map((name) => [name, "…"])));
  const authorization = token ? ` -H 'Authorization: Bearer ${token}'` : "";
  return `curl -X POST${authorization} -H 'Content-Type: application/json' -d '${body}' ${url}`;
}
