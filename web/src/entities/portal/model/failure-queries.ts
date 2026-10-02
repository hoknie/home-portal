import { useQuery } from "@tanstack/react-query";

import { fetchFailure } from "../api/failure";

export const failureKey = ["portal-failure"] as const;

export const FAILURE_POLL_MILLISECONDS = 5_000;

export function useFailure() {
  return useQuery({
    queryKey: failureKey,
    queryFn: fetchFailure,
    retry: false,
    refetchInterval: FAILURE_POLL_MILLISECONDS,
    refetchIntervalInBackground: true,
  });
}
