import { expect, it } from "vitest";

import { automationsKey, automationsSchema, runsSchema, scriptsKey, scriptsSchema } from "@/entities/automation";
import { webhooksKey, webhooksSchema } from "@/entities/webhook";
import { workflowsKey, workflowsSchema } from "@/entities/workflow";
import { apiSamples } from "@/shared/api";
import { renderWithProviders, testQueryClient } from "@/shared/lib/testing";

import { type ItemReferences, useItemReferences } from "./use-item-references";

function referencesWith(editing: boolean) {
  const client = testQueryClient();
  client.setQueryData(automationsKey, { data: automationsSchema.parse(apiSamples.automations), revision: null });
  client.setQueryData(webhooksKey, { data: webhooksSchema.parse(apiSamples.webhooks), revision: null });
  client.setQueryData(workflowsKey, { data: workflowsSchema.parse(apiSamples.workflows), revision: null });
  client.setQueryData(scriptsKey, { ...scriptsSchema.parse(apiSamples.automationScripts), editing });
  const seen: { references: ItemReferences | null } = { references: null };
  function Probe() {
    seen.references = useItemReferences();
    return null;
  }
  renderWithProviders(<Probe />, client);
  return seen.references as ItemReferences;
}

const runs = runsSchema.parse(apiSamples.automationRuns).runs;

it("an automation that ran a workflow names and links both", () => {
  const references = referencesWith(false);
  const run = { ...runs.find((candidate) => candidate.workflow === "revive")!, automation: "restart-media" };
  const { reference, also } = references.sourceOf(run);
  expect(reference).toMatchObject({ kind: "Automation", href: "/admin/automations/edit/?id=restart-media" });
  expect(also).toMatchObject({ kind: "Workflow", href: "/admin/workflows/revive/" });
});

it("a webhook's own run names the webhook and a workflow run without an automation names the workflow", () => {
  const references = referencesWith(false);
  const webhook = apiSamples.webhooks.webhooks[0];
  const own = { ...runs[0], automation: webhook.id, fields: { ...runs[0].fields, "webhook.id": webhook.id }, workflow: null };
  expect(references.sourceOf(own).reference).toMatchObject({ kind: "Webhook", text: webhook.title, href: `/admin/webhooks/details/?id=${webhook.id}` });
  expect(references.sourceOf({ ...runs[0], automation: "workflow:revive", workflow: "revive" }).reference).toMatchObject({ kind: "Workflow", href: "/admin/workflows/revive/" });
});

it("an item that no longer exists shows its id without a link", () => {
  expect(referencesWith(false).sourceOf({ ...runs[0], automation: "gone", workflow: null }).reference).toEqual({ kind: "Automation", text: "gone", href: null });
});

it("a script is linked only while the scripts page is available", () => {
  const target = { run: { script: "backup.sh" }, workflow: null };
  expect(referencesWith(false).actionOf(target)).toMatchObject({ kind: "Script", text: "backup.sh", href: null });
  expect(referencesWith(true).actionOf(target)).toMatchObject({ href: "/admin/scripts/?path=backup.sh" });
});
