import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { fetchDns, saveDns } from "../api/dns";
import type { DnsForm } from "./form";
import type { Dns } from "./schema";

export const dnsKey = ["dns"] as const;

export const DNS_POLL_MILLISECONDS = 5_000;

export function useDns() {
  return useQuery({ queryKey: dnsKey, queryFn: fetchDns, refetchInterval: DNS_POLL_MILLISECONDS });
}

export function useSaveDns() {
  const client = useQueryClient();
  return useMutation({
    mutationFn: ({ form, dns, revision }: { form: DnsForm; dns: Dns; revision: string | null }) => saveDns(form, dns, revision),
    onSuccess: (answer) => client.setQueryData(dnsKey, answer),
    onSettled: () => client.invalidateQueries({ queryKey: dnsKey }),
  });
}
