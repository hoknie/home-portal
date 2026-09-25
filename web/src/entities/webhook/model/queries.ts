import { keepPreviousData, useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { STATUS_REFRESH_MILLISECONDS } from "@/shared/config";

import { type WebhookRequest, createWebhook, deleteWebhook, fetchWebhooks, issueToken, removeToken, updateWebhook } from "../api/webhooks";

export const webhooksKey = ["webhooks"] as const;

export function useWebhooks() {
  return useQuery({ queryKey: webhooksKey, queryFn: fetchWebhooks, refetchInterval: STATUS_REFRESH_MILLISECONDS, placeholderData: keepPreviousData });
}

function useInvalidating<T, R>(run: (input: T) => Promise<R>) {
  const client = useQueryClient();
  return useMutation({
    mutationFn: run,
    onSettled: () => {
      void client.invalidateQueries({ queryKey: webhooksKey });
      void client.invalidateQueries({ queryKey: ["automation-catalogue"] });
    },
  });
}

export function useSaveWebhook() {
  return useInvalidating(({ id, body, revision }: { id: string | null; body: WebhookRequest; revision: string | null }) =>
    id === null
      ? createWebhook(body, revision).then((answer) => (answer.data.token ? { token: answer.data.token, address: answer.data.webhook.address } : null))
      : updateWebhook(id, body, revision).then(() => null),
  );
}

export function useDeleteWebhook() {
  return useInvalidating(({ id, revision }: { id: string; revision: string | null }) => deleteWebhook(id, revision));
}

export function useIssueToken() {
  return useInvalidating(({ id, revision }: { id: string; revision: string | null }) => issueToken(id, revision));
}

export function useRemoveToken() {
  return useInvalidating(({ id, revision }: { id: string; revision: string | null }) => removeToken(id, revision));
}
