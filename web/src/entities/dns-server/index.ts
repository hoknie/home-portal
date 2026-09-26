export { fetchDns, saveDns } from "./api/dns";
export { DNS_PORT_MAX, DNS_PORT_MIN, DNS_TTL_MAX, DNS_TTL_MIN, dnsFormOf, dnsFormSchema, dnsRequestOf } from "./model/form";
export type { DnsForm } from "./model/form";
export { DNS_POLL_MILLISECONDS, dnsKey, useDns, useSaveDns } from "./model/queries";
export { answerIn, dnsSchema } from "./model/schema";
export type { Dns, DnsTransport } from "./model/schema";
