import { keepPreviousData, useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { STATUS_REFRESH_MILLISECONDS } from "@/shared/config";

import {
  type AutomationRequest,
  type RunsFilter,
  createAutomation,
  deleteAutomation,
  fetchAutomations,
  fetchCatalogue,
  fetchRuns,
  fetchSchedule,
  fetchScripts,
  runAutomation,
  updateAutomation,
} from "../api/automations";

export const automationsKey = ["automations"] as const;
export const runsKey = (filter: RunsFilter = {}) =>
  ["automation-runs", filter.automation ?? null, filter.webhook ?? null, filter.text ?? null] as const;
export const catalogueKey = ["automation-catalogue"] as const;
export const scriptsKey = ["automation-scripts"] as const;
export const scheduleKey = (cron: string) => ["automation-schedule", cron] as const;

export function useAutomations() {
  return useQuery({
    queryKey: automationsKey,
    queryFn: fetchAutomations,
    refetchInterval: STATUS_REFRESH_MILLISECONDS,
    placeholderData: keepPreviousData,
  });
}

export const LIVE_RUNS_MILLISECONDS = 5_000;

export function useRuns(filter: RunsFilter = {}, live = true, enabled = true) {
  return useQuery({
    queryKey: runsKey(filter),
    queryFn: () => fetchRuns(filter),
    enabled,
    refetchOnWindowFocus: live,
    refetchInterval: live ? LIVE_RUNS_MILLISECONDS : false,
    placeholderData: keepPreviousData,
  });
}

export function useCatalogue() {
  return useQuery({ queryKey: catalogueKey, queryFn: fetchCatalogue });
}

export function useScripts() {
  return useQuery({ queryKey: scriptsKey, queryFn: fetchScripts });
}

export function useSchedule(cron: string) {
  return useQuery({
    queryKey: scheduleKey(cron),
    queryFn: () => fetchSchedule(cron),
    enabled: cron.trim() !== "",
    retry: false,
    placeholderData: keepPreviousData,
  });
}

function useInvalidating<T, R>(run: (input: T) => Promise<R>) {
  const client = useQueryClient();
  return useMutation({
    mutationFn: run,
    onSettled: () => {
      void client.invalidateQueries({ queryKey: automationsKey });
      void client.invalidateQueries({ queryKey: ["automation-runs"] });
    },
  });
}

export function useSaveAutomation() {
  return useInvalidating(({ id, body, revision }: { id: string | null; body: AutomationRequest; revision: string | null }) =>
    id === null ? createAutomation(body, revision) : updateAutomation(id, body, revision),
  );
}

export function useDeleteAutomation() {
  return useInvalidating(({ id, revision }: { id: string; revision: string | null }) => deleteAutomation(id, revision));
}

export function useRunAutomation() {
  return useInvalidating((id: string) => runAutomation(id));
}
