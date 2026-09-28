import { keepPreviousData, useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { STATUS_REFRESH_MILLISECONDS } from "@/shared/config";

import { createWorkflow, deleteWorkflow, fetchPortalValues, fetchSecretNames, fetchWorkflowCatalogue, fetchWorkflows, runWorkflow, updateWorkflow } from "../api/workflows";
import type { WorkflowRequest } from "./schema";

export const workflowsKey = ["workflows"] as const;
export const workflowCatalogueKey = ["workflow-catalogue"] as const;
export const secretNamesKey = ["secret-names"] as const;
export const portalValuesKey = ["workflow-portal"] as const;

export const PORTAL_REFRESH_MILLISECONDS = 30_000;

export function usePortalValues() {
  return useQuery({ queryKey: portalValuesKey, queryFn: fetchPortalValues, refetchInterval: PORTAL_REFRESH_MILLISECONDS });
}

export function useSecretNames() {
  return useQuery({ queryKey: secretNamesKey, queryFn: fetchSecretNames });
}

export function useWorkflows() {
  return useQuery({
    queryKey: workflowsKey,
    queryFn: fetchWorkflows,
    refetchInterval: STATUS_REFRESH_MILLISECONDS,
    placeholderData: keepPreviousData,
  });
}

export function useWorkflowCatalogue() {
  return useQuery({ queryKey: workflowCatalogueKey, queryFn: fetchWorkflowCatalogue, staleTime: Infinity });
}

function useInvalidating<T, R>(run: (input: T) => Promise<R>) {
  const client = useQueryClient();
  return useMutation({
    mutationFn: run,
    onSettled: () => {
      void client.invalidateQueries({ queryKey: workflowsKey });
      void client.invalidateQueries({ queryKey: ["automations"] });
      void client.invalidateQueries({ queryKey: ["automation-runs"] });
    },
  });
}

export function useSaveWorkflow() {
  return useInvalidating(({ id, body, revision }: { id: string | null; body: WorkflowRequest; revision: string | null }) =>
    id === null ? createWorkflow(body, revision) : updateWorkflow(id, body, revision),
  );
}

export function useDeleteWorkflow() {
  return useInvalidating(({ id, revision }: { id: string; revision: string | null }) => deleteWorkflow(id, revision));
}

export function useRunWorkflow() {
  return useInvalidating(({ id, inputs }: { id: string; inputs: Record<string, unknown> }) => runWorkflow(id, inputs));
}
