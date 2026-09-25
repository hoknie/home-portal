import { request } from "@/shared/api";
import { api } from "@/shared/config";

import { createdWebhookSchema, tokenSchema, webhookSchema, webhooksSchema } from "../model/schema";

export type WebhookRequest = {
  title: string;
  enabled: boolean;
  tags: string[];
  variables: string[];
  action: "event" | "script";
  run: { script: string; args: string[]; timeout_seconds: number } | null;
  with_token?: boolean;
};

export function fetchWebhooks() {
  return request(api.webhooks, { schema: webhooksSchema });
}

export function createWebhook(body: WebhookRequest, revision: string | null) {
  return request(api.webhooks, { method: "POST", body, revision, schema: createdWebhookSchema });
}

export function updateWebhook(id: string, body: WebhookRequest, revision: string | null) {
  return request(api.webhook(id), { method: "PUT", body, revision, schema: webhookSchema });
}

export function deleteWebhook(id: string, revision: string | null) {
  return request(api.webhook(id), { method: "DELETE", revision, schema: webhooksSchema });
}

export async function issueToken(id: string, revision: string | null) {
  return (await request(api.webhookToken(id), { method: "POST", body: {}, revision, schema: tokenSchema })).data.token;
}

export function removeToken(id: string, revision: string | null) {
  return request(api.webhookToken(id), { method: "DELETE", revision, schema: webhooksSchema });
}
