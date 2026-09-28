import { request } from "@/shared/api";
import { api } from "@/shared/config";

import { type WorkflowRequest, secretNamesSchema, workflowCatalogueSchema, workflowRunSchema, workflowSchema, workflowsSchema } from "../model/schema";

export function fetchWorkflows() {
  return request(api.workflows, { schema: workflowsSchema });
}

export function createWorkflow(body: WorkflowRequest, revision: string | null) {
  return request(api.workflows, { method: "POST", body, revision, schema: workflowSchema });
}

export function updateWorkflow(id: string, body: WorkflowRequest, revision: string | null) {
  return request(api.workflow(id), { method: "PUT", body, revision, schema: workflowSchema });
}

export function deleteWorkflow(id: string, revision: string | null) {
  return request(api.workflow(id), { method: "DELETE", revision, schema: workflowsSchema });
}

export async function runWorkflow(id: string, inputs: Record<string, unknown>) {
  return (await request(api.workflowRun(id), { method: "POST", body: { inputs }, schema: workflowRunSchema })).data;
}

export async function fetchWorkflowCatalogue() {
  return (await request(api.workflowCatalogue, { schema: workflowCatalogueSchema })).data;
}

export async function fetchSecretNames() {
  return (await request(api.secrets, { schema: secretNamesSchema })).data.secrets;
}
