import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { fetchModules, switchModule } from "../api/modules";
import type { ModuleName } from "./schema";

export const modulesKey = ["modules"] as const;

export function useModules() {
  return useQuery({ queryKey: modulesKey, queryFn: fetchModules });
}

export function useSwitchModule() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: ({ name, enabled, revision }: { name: ModuleName; enabled: boolean; revision: string | null }) =>
      switchModule(name, enabled, revision),
    onSuccess: (answer) => client.setQueryData(modulesKey, answer),
    onSettled: () => client.invalidateQueries(),
  });
}
