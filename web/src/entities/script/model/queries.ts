import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { createFolder, createScript, deleteFolder, deleteScript, fetchScript, fetchScripts, moveScript, saveScript } from "../api/scripts";

export const scriptsTreeKey = ["scripts", "tree"] as const;

export const scriptKey = (path: string) => ["scripts", "file", path] as const;

export function useScriptTree({ enabled = true }: { enabled?: boolean } = {}) {
  return useQuery({ queryKey: scriptsTreeKey, queryFn: fetchScripts, enabled });
}

export function useScriptText(path: string | null) {
  return useQuery({
    queryKey: scriptKey(path ?? ""),
    queryFn: () => fetchScript(path ?? ""),
    enabled: path !== null,
    staleTime: Number.POSITIVE_INFINITY,
  });
}

function useChanging<T, R>(run: (input: T) => Promise<R>) {
  const client = useQueryClient();
  return useMutation({
    mutationFn: run,
    onSettled: () => {
      void client.invalidateQueries({ queryKey: ["scripts"] });
    },
  });
}

export function useSaveScript() {
  return useChanging(({ path, content, revision }: { path: string; content: string; revision: string }) => saveScript(path, content, revision));
}

export function useCreateScript() {
  return useChanging(({ path, content }: { path: string; content: string }) => createScript(path, content));
}

export function useDeleteScript() {
  return useChanging(({ path, revision }: { path: string; revision: string }) => deleteScript(path, revision));
}

export function useMoveScript() {
  return useChanging(({ from, to, revision }: { from: string; to: string; revision: string }) => moveScript(from, to, revision));
}

export function useCreateFolder() {
  return useChanging((name: string | null) => createFolder(name));
}

export function useDeleteFolder() {
  return useChanging((name: string) => deleteFolder(name));
}
