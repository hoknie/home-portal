import { keepPreviousData, useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { STATUS_REFRESH_MILLISECONDS } from "@/shared/config";

import { changeChannel, changeRules, fetchNotifications, sendTest } from "../api/notifications";
import type { Rules } from "./schema";

export const notificationsKey = ["notifications"] as const;

export function useNotifications({ live = false }: { live?: boolean } = {}) {
  return useQuery({
    queryKey: notificationsKey,
    queryFn: fetchNotifications,
    refetchInterval: live ? STATUS_REFRESH_MILLISECONDS : false,
    placeholderData: keepPreviousData,
  });
}

function useInvalidating<T, R>(run: (input: T) => Promise<R>) {
  const client = useQueryClient();
  return useMutation({
    mutationFn: run,
    onSettled: () => {
      void client.invalidateQueries({ queryKey: notificationsKey });
    },
  });
}

export function useChangeRules() {
  return useInvalidating(({ rules, revision }: { rules: Rules; revision: string | null }) => changeRules(rules, revision));
}

export function useChangeChannel() {
  return useInvalidating(({ name, settings, revision }: { name: string; settings: Record<string, unknown>; revision: string | null }) =>
    changeChannel(name, settings, revision),
  );
}

export function useSendTest() {
  return useInvalidating((channel: string) => sendTest(channel));
}
