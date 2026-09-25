import { expect, it } from "vitest";

import { apiSamples } from "@/shared/api";

import { acceptedSchema, createdWebhookSchema, curlExample, shortAddress, tokenSchema, webhooksSchema } from "./schema";

it("the webhook samples parse", () => {
  const listed = webhooksSchema.parse(apiSamples.webhooks);
  expect(listed.webhooks.map((webhook) => webhook.action)).toEqual(["script", "event"]);
  expect(listed.webhooks[0]).toMatchObject({ protected: true, variables: ["branch", "commit"], last_received: { status: 202 } });
  expect(createdWebhookSchema.parse(apiSamples.webhookCreated).token).toHaveLength(64);
  expect(acceptedSchema.parse(apiSamples.webhookAccepted).run_id).toBe("42");
  expect(tokenSchema.parse(apiSamples.webhookToken).token).toHaveLength(64);
});

it("an address is shortened to the start and the end of its id", () => {
  expect(shortAddress("/webhook/7d3f2a4e-5b1c-4e8f-9a2d-6c0b1e3f4a5d")).toBe("/webhook/7d3f2a4e…4a5d");
  expect(shortAddress("/webhook/short")).toBe("/webhook/short");
});

it("the curl example carries the token and every variable", () => {
  expect(curlExample("https://portal/webhook/x", "t0k", ["branch"])).toBe(
    `curl -X POST -H 'Authorization: Bearer t0k' -H 'Content-Type: application/json' -d '{"branch":"…"}' https://portal/webhook/x`,
  );
  expect(curlExample("https://portal/webhook/x", null, [])).toBe(`curl -X POST -H 'Content-Type: application/json' -d '{}' https://portal/webhook/x`);
});
