import { request } from "@/shared/api";
import { api } from "@/shared/config";

import { type DnsForm, dnsRequestOf } from "../model/form";
import { type Dns, dnsSchema } from "../model/schema";

export function fetchDns() {
  return request(api.dns, { schema: dnsSchema });
}

export function saveDns(form: DnsForm, dns: Dns, revision: string | null) {
  return request(api.dns, { method: "PUT", body: dnsRequestOf(form, dns), revision, schema: dnsSchema });
}
