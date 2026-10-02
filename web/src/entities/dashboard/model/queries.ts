import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { fetchDashboard } from "../api/dashboard";
import { fetchLayout, saveLayout } from "../api/layout";
import { deleteLibraryWidget, fetchLibrary, saveLibraryWidget } from "../api/library";
import type { LibraryEntry } from "./library";
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
      void client.invalidateQueries({ queryKey: libraryKey });
    },
  });
}

export const libraryKey = ["dashboard", "library"] as const;

export function useLibrary() {
  return useQuery({ queryKey: libraryKey, queryFn: fetchLibrary, staleTime: 10_000 });
}

export function useSaveLibraryWidget() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: ({ entry, existing, revision }: { entry: LibraryEntry; existing: string | null; revision: string | null }) => saveLibraryWidget(entry, existing, revision),
    onSuccess: () => {
      void client.invalidateQueries({ queryKey: libraryKey });
      void client.invalidateQueries({ queryKey: dashboardKey, exact: true });
    },
  });
}

export function useDeleteLibraryWidget() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: ({ id, revision }: { id: string; revision: string | null }) => deleteLibraryWidget(id, revision),
    onSuccess: () => void client.invalidateQueries({ queryKey: libraryKey }),
  });
}
