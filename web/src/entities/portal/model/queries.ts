import { useQuery } from "@tanstack/react-query";

import { fetchPortal } from "../api/portal";

export const portalKey = ["public-portal"] as const;

export function usePortal() {
  return useQuery({ queryKey: portalKey, queryFn: fetchPortal, retry: false, staleTime: 30_000 });
}
