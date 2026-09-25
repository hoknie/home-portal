import { z } from "zod";

export const PROXY_TLS_MODES = ["acme", "internal", "files"] as const;

export const proxyTlsModeSchema = z.enum(PROXY_TLS_MODES).catch("acme");

export const proxyRouteSchema = z.object({
  host: z.string(),
  address: z.string(),
  service: z.string().nullable(),
  upstream: z.string(),
  tls: proxyTlsModeSchema,
  auth: z.array(z.string()).default([]),
  environments: z.array(z.string()).default([]),
});

export type ProxyRoute = z.infer<typeof proxyRouteSchema>;

export const DOWNLOAD_STAGES = ["idle", "downloading", "installed", "failed"] as const;

export const caddySchema = z.object({
  managed: z.boolean(),
  installed: z.string().nullable(),
  installed_from: z.string().nullable().default(null),
  source: z.string(),
  version: z.string(),
  release_url: z.string(),
  platform: z.string().nullable(),
  platform_error: z.string().nullable().default(null),
  download: z.object({ state: z.enum(DOWNLOAD_STAGES).catch("idle"), error: z.string().nullable() }),
  log: z.array(z.string()).default([]),
});

export type Caddy = z.infer<typeof caddySchema>;

export const proxySettingsSchema = z.object({
  http_port: z.number().default(80),
  https_port: z.number().default(443),
  portal_host: z.string().nullable(),
  cookie_domain: z.string().nullable(),
  tls: z.object({
    mode: proxyTlsModeSchema,
    email: z.string().nullable().default(null),
    certificate: z.string().nullable().default(null),
    key: z.string().nullable().default(null),
  }),
});

export type ProxySettings = z.infer<typeof proxySettingsSchema>;

export const proxySchema = z.object({
  enabled: z.boolean(),
  settings: proxySettingsSchema,
  admin: z.string(),
  reachable: z.boolean(),
  in_sync: z.boolean(),
  last_applied_at: z.string().nullable(),
  last_error: z.string().nullable(),
  routes: z.array(proxyRouteSchema),
  caddy: caddySchema,
});

export type Proxy = z.infer<typeof proxySchema>;

export function usesInternal(proxy: Proxy) {
  return proxy.routes.some((route) => route.tls === "internal");
}
