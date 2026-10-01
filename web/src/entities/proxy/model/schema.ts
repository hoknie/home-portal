import type { z } from "zod";

import { generated } from "@/shared/api";

const served = generated.proxy;

export const proxyTlsModeSchema = served.tlsModeSchema;

export const PROXY_TLS_MODES = proxyTlsModeSchema.unwrap().options;

export const proxyRouteSchema = served.routeResponseSchema;

export type ProxyRoute = z.infer<typeof proxyRouteSchema>;

export const DOWNLOAD_STAGES = served.downloadStageSchema.unwrap().options;

export const caddySchema = served.caddyResponseSchema;

export type Caddy = z.infer<typeof caddySchema>;

export const proxySettingsSchema = served.settingsResponseSchema;

export type ProxySettings = z.infer<typeof proxySettingsSchema>;

export const proxySchema = served.proxyResponseSchema;

export type Proxy = z.infer<typeof proxySchema>;

export function usesInternal(proxy: Proxy) {
  return proxy.routes.some((route) => route.tls === "internal");
}
