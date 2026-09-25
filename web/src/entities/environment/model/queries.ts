import { useQuery } from "@tanstack/react-query";

import { fetchEnvironment } from "../api/environment";

export const environmentKey = ["environment"] as const;

export function useEnvironment() {
  return useQuery({ queryKey: environmentKey, queryFn: fetchEnvironment, staleTime: 300_000, retry: false });
}
