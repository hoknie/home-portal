import { z } from "zod";

export const WEBHOOK_ACTIONS = ["event", "script"] as const;

export const webhookSchema = z.object({
  id: z.string(),
  title: z.string(),
  enabled: z.boolean(),
  tags: z.array(z.string()).default([]),
  address: z.string(),
  protected: z.boolean(),
  variables: z.array(z.string()),
  action: z.enum(WEBHOOK_ACTIONS).catch("event"),
  run: z.object({ script: z.string(), args: z.array(z.string()), timeout_seconds: z.number() }).nullable(),
  last_received: z.object({ at: z.string(), status: z.number() }).nullable(),
});

export type Webhook = z.infer<typeof webhookSchema>;

export const webhooksSchema = z.object({ webhooks: z.array(webhookSchema) });

export const createdWebhookSchema = z.object({ webhook: webhookSchema, token: z.string().nullable() });

export const tokenSchema = z.object({ token: z.string() });

export const acceptedSchema = z.object({ accepted: z.boolean(), run_id: z.string().optional() });

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
