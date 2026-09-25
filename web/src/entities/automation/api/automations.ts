import { request } from "@/shared/api";
import { api } from "@/shared/config";

import { automationSchema, automationsSchema, catalogueSchema, queuedSchema, runsSchema, scheduleSchema, scriptsSchema } from "../model/schema";

export type AutomationRequest = {
  id: string;
  title: string;
  enabled: boolean;
  tags: string[];
  cooldown_seconds: number;
  when: Record<string, unknown>;
  run: { script: string; args: string[]; timeout_seconds: number };
};

export function fetchAutomations() {
  return request(api.automations, { schema: automationsSchema });
}

export function createAutomation(body: AutomationRequest, revision: string | null) {
  return request(api.automations, { method: "POST", body, revision, schema: automationSchema });
}

export function updateAutomation(id: string, body: AutomationRequest, revision: string | null) {
  return request(api.automation(id), { method: "PUT", body, revision, schema: automationSchema });
}

export function deleteAutomation(id: string, revision: string | null) {
  return request(api.automation(id), { method: "DELETE", revision, schema: automationsSchema });
}

export async function runAutomation(id: string) {
  return (await request(api.automationRun(id), { method: "POST", body: {}, schema: queuedSchema })).data;
}

export type RunsFilter = { automation?: string | null; webhook?: string | null; text?: string | null };

export async function fetchRuns(filter: RunsFilter) {
  return (await request(api.automationRuns(filter), { schema: runsSchema })).data;
}

export async function fetchCatalogue() {
  return (await request(api.automationCatalogue, { schema: catalogueSchema })).data;
}

export async function fetchScripts() {
  return (await request(api.automationScripts, { schema: scriptsSchema })).data;
}

export async function fetchSchedule(cron: string) {
  return (await request(api.automationSchedule(cron), { schema: scheduleSchema })).data;
}
