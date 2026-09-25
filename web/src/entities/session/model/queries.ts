import { useQuery } from "@tanstack/react-query";

import { fetchSession } from "../api/session";

export const sessionKey = ["session"] as const;

export function useSession() {
  return useQuery({ queryKey: sessionKey, queryFn: fetchSession, retry: false, staleTime: 60_000 });
}
