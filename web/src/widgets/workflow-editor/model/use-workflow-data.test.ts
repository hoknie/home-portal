import { expect, it } from "vitest";

import { automationsSchema, catalogueSchema } from "@/entities/automation";
import { workflowsSchema } from "@/entities/workflow";
import { apiSamples } from "@/shared/api";

import { eventFieldsFor, eventKnowledgeFor } from "./use-workflow-data";

const catalogue = catalogueSchema.parse(apiSamples.automationCatalogue);
const automations = automationsSchema.parse(apiSamples.automations).automations;
const revive = workflowsSchema.parse(apiSamples.workflows).workflows[0];

it("a workflow a webhook starts offers the short event names, the body and the webhook's variables", () => {
  const names = eventFieldsFor(revive, automations, catalogue).map((field) => field.name);
  expect(names.slice(0, 2)).toEqual(["name", "at"]);
  expect(names).toContain("webhook.body");
  expect(names).toContain("webhook.service");
  expect(names).not.toContain("event.name");
});

it("the event knowledge lists the variables only for a workflow a webhook starts", () => {
  expect(eventKnowledgeFor(revive, automations, catalogue).variables).toEqual(["service"]);
  expect(eventKnowledgeFor({ ...revive, used_by: [] }, automations, catalogue).variables).toBeNull();
  expect(eventKnowledgeFor(revive, automations, catalogue).fields).toContain("service.id");
});
