import { z } from "zod";

import { PROXY_TLS_MODES, type Proxy } from "./schema";

const HOST_LABEL = /^[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?$/;

export function proxyHostAccepted(value: string) {
  return value.length <= 253 && value.split(".").every((label) => HOST_LABEL.test(label));
}

export function within(host: string, domain: string) {
  return host === domain || host.endsWith(`.${domain}`);
}

export const proxySettingsFormSchema = z
  .object({
    enabled: z.boolean(),
    http_port: z.number({ error: "validation.port" }).int("validation.port").min(1, "validation.port").max(65535, "validation.port"),
    https_port: z.number({ error: "validation.port" }).int("validation.port").min(1, "validation.port").max(65535, "validation.port"),
    portal_host: z.string().trim().toLowerCase(),
    cookie_domain: z.string().trim().toLowerCase(),
    mode: z.enum(PROXY_TLS_MODES),
    email: z.string().trim(),
    certificate: z.string().trim(),
    key: z.string().trim(),
  })
  .refine((form) => form.http_port !== form.https_port, { path: ["https_port"], message: "validation.proxySamePorts" })
  .refine((form) => form.portal_host === "" || proxyHostAccepted(form.portal_host), { path: ["portal_host"], message: "validation.proxyHost" })
  .refine((form) => !form.enabled || form.portal_host !== "", { path: ["portal_host"], message: "validation.proxyPortalHostRequired" })
  .refine((form) => form.cookie_domain === "" || proxyHostAccepted(form.cookie_domain), { path: ["cookie_domain"], message: "validation.proxyHost" })
  .refine((form) => form.cookie_domain === "" || form.portal_host === "" || within(form.portal_host, form.cookie_domain), {
    path: ["cookie_domain"],
    message: "validation.proxyCookieDomain",
  })
  .refine((form) => form.email === "" || /^[^@\s]+@[^@\s]+\.[^@\s]+$/.test(form.email), { path: ["email"], message: "validation.proxyEmail" })
  .refine((form) => form.mode !== "files" || form.certificate !== "", { path: ["certificate"], message: "validation.proxyFile" })
  .refine((form) => form.mode !== "files" || form.key !== "", { path: ["key"], message: "validation.proxyFile" });

export type ProxySettingsForm = z.infer<typeof proxySettingsFormSchema>;

export function proxySettingsFormOf(proxy: Proxy): ProxySettingsForm {
  const { settings } = proxy;
  return {
    enabled: proxy.enabled,
    http_port: settings.http_port,
    https_port: settings.https_port,
    portal_host: settings.portal_host ?? "",
    cookie_domain: settings.cookie_domain ?? "",
    mode: settings.tls.mode,
    email: settings.tls.email ?? "",
    certificate: settings.tls.certificate ?? "",
    key: settings.tls.key ?? "",
  };
}

export function proxySettingsRequestOf(form: ProxySettingsForm) {
  const blank = (value: string) => (value.trim() === "" ? null : value.trim());
  return {
    enabled: form.enabled,
    http_port: form.http_port,
    https_port: form.https_port,
    portal_host: blank(form.portal_host),
    cookie_domain: blank(form.cookie_domain),
    tls: {
      mode: form.mode,
      email: blank(form.email),
      certificate: form.mode === "files" ? blank(form.certificate) : null,
      key: form.mode === "files" ? blank(form.key) : null,
    },
  };
}

export const CADDY_LATEST = "latest";

const LOOPBACK_HOSTS = new Set(["localhost", "127.0.0.1", "[::1]"]);
const CADDY_VERSION = /^v?\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$/;

export function caddySourceAccepted(value: string) {
  if (!URL.canParse(value)) {
    return false;
  }
  const url = new URL(value);
  return url.protocol === "https:" || (url.protocol === "http:" && (LOOPBACK_HOSTS.has(url.hostname) || url.hostname.startsWith("127.")));
}

export const caddySourceFormSchema = z.object({
  source: z.string().trim().refine((value) => value === "" || caddySourceAccepted(value), "validation.caddySource"),
  version: z
    .string()
    .trim()
    .refine((value) => value === "" || value === CADDY_LATEST || CADDY_VERSION.test(value), "validation.caddyVersion"),
});

export type CaddySourceForm = z.infer<typeof caddySourceFormSchema>;

export function caddySourceFormOf(proxy: Proxy): CaddySourceForm {
  return { source: proxy.caddy.source, version: proxy.caddy.version };
}
