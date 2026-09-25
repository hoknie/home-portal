import { useQuery } from "@tanstack/react-query";

import { fetchWidgetData } from "../api/widget";

export const widgetKey = (id: string, scope: "private" | "public") => ["widget", scope, id] as const;

export function useWidgetData(id: string, scope: "private" | "public" = "private") {
  return useQuery({
    queryKey: widgetKey(id, scope),
    queryFn: async () => fetchWidgetData(id, scope),
    refetchInterval: (query) => (query.state.data ? query.state.data.refresh_seconds * 1000 : false),
    retry: false,
  });
}
