import { keepPreviousData, useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { STATUS_REFRESH_MILLISECONDS } from "@/shared/config";

import { createService, deleteService, fetchHistory, fetchServices, probeService, updateService } from "../api/services";
import type { HistoryRange } from "./history";
import type { ServiceForm } from "./form";

export const servicesKey = ["services"] as const;

export const HISTORY_REFRESH_MILLISECONDS = 60_000;

export const historyKey = (id: string, range: HistoryRange) => ["service-history", id, range] as const;

export function useServices() {
  return useQuery({
    queryKey: servicesKey,
    queryFn: fetchServices,
    refetchInterval: STATUS_REFRESH_MILLISECONDS,
    placeholderData: keepPreviousData,
  });
}

export function useSaveService() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: ({ id, form, revision }: { id: string | null; form: ServiceForm; revision: string | null }) =>
      id === null ? createService(form, revision) : updateService(id, form, revision),
    onSettled: () => client.invalidateQueries({ queryKey: servicesKey }),
  });
}

export function useDeleteService() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: ({ id, revision }: { id: string; revision: string | null }) => deleteService(id, revision),
    onSettled: () => client.invalidateQueries({ queryKey: servicesKey }),
  });
}

export function useServiceHistory(id: string, range: HistoryRange) {
  return useQuery({
    queryKey: historyKey(id, range),
    queryFn: () => fetchHistory(id, range),
    refetchInterval: HISTORY_REFRESH_MILLISECONDS,
    placeholderData: keepPreviousData,
  });
}

export function useProbeService() {
  return useMutation({ mutationFn: (id: string) => probeService(id) });
}
