import { describe, expect, it } from "vitest";

import { catalogueSchema } from "@/entities/automation";
import { webhooksSchema } from "@/entities/webhook";
import { apiSamples } from "@/shared/api";

import { emptyWebhookForm, webhookEventOf, webhookFormOf, webhookFormSchema, webhookRequestOf } from "./webhook-form";

const catalogue = catalogueSchema.parse(apiSamples.automationCatalogue);
const fieldsOf = (variables: string[]) => webhookEventOf(catalogue, variables).fields.map((field) => field.name);

describe("webhook form", () => {
  it("a sample webhook round-trips into its request", () => {
    const deploy = webhooksSchema.parse(apiSamples.webhooks).webhooks[0];
    expect(webhookRequestOf(webhookFormOf(deploy), false)).toEqual({
      title: "Deploy from CI",
      enabled: true,
      tags: ["ci", "media"],
      variables: ["branch", "commit"],
      action: "script",
      run: { script: "deploy.sh", args: ["--", "{{webhook.branch}}"], timeout_seconds: 300 },
    });
  });

  it("mirrors the server's rules", () => {
    const schema = webhookFormSchema(fieldsOf);
    const valid = { ...emptyWebhookForm, title: "Deploy", variables: ["branch"], script: "deploy.sh", args: [{ value: "{{webhook.branch}}" }] };
    const issues = (form: typeof valid) => (schema.safeParse(form).error?.issues ?? []).map((issue) => issue.path.join("."));
    expect(issues(valid)).toEqual([]);
    expect(issues({ ...valid, variables: ["Branch"], args: [] })).toEqual(["variables"]);
    expect(issues({ ...valid, args: [{ value: "{{webhook.commit}}" }] })).toEqual(["args.0.value"]);
    expect(issues({ ...valid, script: "../x" })).toEqual(["script"]);
    expect(issues({ ...valid, action: "event", script: "" })).toEqual([]);
  });

  it("an event webhook sends no run, and only a new one asks for a token", () => {
    const form = { ...emptyWebhookForm, title: "Motion", action: "event" as const };
    expect(webhookRequestOf(form, true)).toMatchObject({ run: null, with_token: true });
    expect(webhookRequestOf(form, false)).not.toHaveProperty("with_token");
  });
});
