import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { fetchNetwork, saveNetwork } from "../api/network";
import type { NetworkForm } from "./form";

export const networkKey = ["network"] as const;

export function useNetwork() {
  return useQuery({ queryKey: networkKey, queryFn: fetchNetwork });
}

export function useSaveNetwork() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: ({ form, revision }: { form: NetworkForm; revision: string | null }) => saveNetwork(form, revision),
    onSuccess: (saved) => client.setQueryData(networkKey, saved),
    onError: () => client.invalidateQueries({ queryKey: networkKey }),
  });
}
