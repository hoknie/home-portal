import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { fetchDashboard } from "../api/dashboard";
import { fetchLayout, saveLayout } from "../api/layout";
import type { LayoutRequest } from "./layout";

export const dashboardKey = ["dashboard"] as const;

export const layoutKey = ["dashboard", "layout"] as const;

export function useDashboard() {
  return useQuery({ queryKey: dashboardKey, queryFn: fetchDashboard, staleTime: 30_000 });
}

export function useLayout() {
  return useQuery({ queryKey: layoutKey, queryFn: fetchLayout, staleTime: Infinity, refetchOnWindowFocus: false });
}

export function useSaveLayout() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: ({ layout, revision }: { layout: LayoutRequest; revision: string | null }) => saveLayout(layout, revision),
    onSuccess: (saved) => {
      client.setQueryData(layoutKey, saved);
      void client.invalidateQueries({ queryKey: dashboardKey, exact: true });
    },
  });
}
