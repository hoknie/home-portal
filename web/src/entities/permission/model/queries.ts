import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { fetchPermissions, requestPermissions } from "../api/permissions";
import { type Permissions, anyPending } from "./schema";

export const permissionsKey = ["permissions"] as const;

export const PENDING_REFRESH_MILLISECONDS = 2_000;

export function pendingRefresh(permissions: Permissions | undefined): number | false {
  return anyPending(permissions) ? PENDING_REFRESH_MILLISECONDS : false;
}

export function usePermissions() {
  return useQuery({
    queryKey: permissionsKey,
    queryFn: fetchPermissions,
    refetchInterval: (query) => pendingRefresh(query.state.data?.data),
  });
}

export function useRequestPermissions() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: requestPermissions,
    onSettled: () => client.invalidateQueries({ queryKey: permissionsKey }),
  });
}
