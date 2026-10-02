import { useQuery } from "@tanstack/react-query";

import { fetchWidgetData } from "../api/widget";
import { isPending } from "./schema";

export const RETRY_MILLISECONDS = 30_000;

export const REFRESHING_MILLISECONDS = 5_000;

export const widgetKey = (id: string, scope: "private" | "public") => ["widget", scope, id] as const;

export function useWidgetData(id: string, scope: "private" | "public" = "private") {
  return useQuery({
    queryKey: widgetKey(id, scope),
    queryFn: async () => fetchWidgetData(id, scope),
    refetchInterval: (query) => {
      const answer = query.state.data;
      if (!answer) {
        return RETRY_MILLISECONDS;
      }
      return isPending(answer) || answer.refreshing ? REFRESHING_MILLISECONDS : answer.refresh_seconds * 1000;
    },
    retry: false,
  });
}
