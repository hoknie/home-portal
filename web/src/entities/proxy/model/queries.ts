import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { applyProxy, downloadCaddy, fetchProxy, saveCaddySource, saveProxySettings, startCaddy, stopCaddy } from "../api/proxy";
import type { CaddySourceForm, ProxySettingsForm } from "./form";

export const proxyKey = ["proxy"] as const;

export const CADDY_POLL_MILLISECONDS = 1_500;

export function useProxy() {
  return useQuery({
    queryKey: proxyKey,
    queryFn: fetchProxy,
    refetchInterval: (query) => (query.state.data?.data.caddy.download.state === "downloading" ? CADDY_POLL_MILLISECONDS : false),
  });
}

function useProxyMutation<T>(run: (input: T) => ReturnType<typeof fetchProxy>) {
  const client = useQueryClient();
  return useMutation({
    mutationFn: run,
    onSuccess: (answer) => client.setQueryData(proxyKey, answer),
    onSettled: () => client.invalidateQueries({ queryKey: proxyKey }),
  });
}

export function useApplyProxy() {
  return useProxyMutation<void>(() => applyProxy());
}

export function useDownloadCaddy() {
  return useProxyMutation<void>(() => downloadCaddy());
}

export function useStartCaddy() {
  return useProxyMutation<string | null>(startCaddy);
}

export function useStopCaddy() {
  return useProxyMutation<string | null>(stopCaddy);
}

export function useSaveProxySettings() {
  return useProxyMutation<{ form: ProxySettingsForm; revision: string | null }>(({ form, revision }) => saveProxySettings(form, revision));
}

export function useSaveCaddySource() {
  return useProxyMutation<{ form: CaddySourceForm; revision: string | null }>(({ form, revision }) => saveCaddySource(form, revision));
}
