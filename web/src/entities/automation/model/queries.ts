import { keepPreviousData, useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { STATUS_REFRESH_MILLISECONDS } from "@/shared/config";

import {
  type AutomationRequest,
  type RunsFilter,
  createAutomation,
  deleteAutomation,
  fetchAutomations,
  fetchCatalogue,
  fetchRun,
  fetchRuns,
  fetchSchedule,
  fetchScripts,
  runAutomation,
  stopRun,
  updateAutomation,
} from "../api/automations";
import { type Run, isActive } from "./schema";

export const automationsKey = ["automations"] as const;
export const runsKey = (filter: RunsFilter = {}) =>
  ["automation-runs", filter.automation ?? null, filter.webhook ?? null, filter.text ?? null] as const;
export const runKey = (id: string | null) => ["automation-run", id] as const;
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
export const ACTIVE_RUNS_MILLISECONDS = 1_000;

export function runsRefreshInterval(runs: Run[] | undefined, live: boolean) {
  if (!live) {
    return false;
  }
  return runs?.some(isActive) ? ACTIVE_RUNS_MILLISECONDS : LIVE_RUNS_MILLISECONDS;
}

export function runRefreshInterval(run: Run | undefined) {
  return run === undefined || isActive(run) ? ACTIVE_RUNS_MILLISECONDS : false;
}

export function useRuns(filter: RunsFilter = {}, live = true, enabled = true) {
  return useQuery({
    queryKey: runsKey(filter),
    queryFn: () => fetchRuns(filter),
    enabled,
    refetchOnWindowFocus: live,
    refetchInterval: (query) => runsRefreshInterval(query.state.data?.runs, live),
    placeholderData: keepPreviousData,
  });
}

export function useRun(id: string | null) {
  return useQuery({
    queryKey: runKey(id),
    queryFn: () => fetchRun(id ?? ""),
    enabled: id !== null,
    refetchInterval: (query) => runRefreshInterval(query.state.data),
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
      void client.invalidateQueries({ queryKey: ["automation-run"] });
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

export function useStopRun() {
  return useInvalidating((id: string) => stopRun(id));
}
