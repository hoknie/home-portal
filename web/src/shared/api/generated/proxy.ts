import { z } from "zod";

export const downloadStageSchema = z.enum(["idle", "downloading", "installed", "failed"]).catch("idle");

export type DownloadStage = z.infer<typeof downloadStageSchema>;

export const downloadStateSchema = z.object({ "error": z.string().nullable(), "state": downloadStageSchema });

export type DownloadState = z.infer<typeof downloadStateSchema>;

export const caddyResponseSchema = z.object({ "download": downloadStateSchema, "installed": z.string().nullable(), "installed_from": z.string().nullable(), "log": z.array(z.string()), "managed": z.boolean(), "platform": z.string().nullable(), "platform_error": z.string().nullable(), "release_url": z.string(), "source": z.string(), "version": z.string() });

export type CaddyResponse = z.infer<typeof caddyResponseSchema>;

export const tlsModeSchema = z.enum(["acme", "internal", "files"]).catch("acme");

export type TlsMode = z.infer<typeof tlsModeSchema>;

export const routeResponseSchema = z.object({ "address": z.string(), "auth": z.array(z.string()), "environments": z.array(z.string()), "host": z.string(), "service": z.string().nullable(), "tls": tlsModeSchema, "upstream": z.string() });

export type RouteResponse = z.infer<typeof routeResponseSchema>;

export const tlsPolicySchema = z.object({ "certificate": z.string().nullable(), "email": z.string().nullable(), "key": z.string().nullable(), "mode": tlsModeSchema });

export type TlsPolicy = z.infer<typeof tlsPolicySchema>;

export const settingsResponseSchema = z.object({ "cookie_domain": z.string().nullable(), "http_port": z.number(), "https_port": z.number(), "portal_host": z.string().nullable(), "tls": tlsPolicySchema });

export type SettingsResponse = z.infer<typeof settingsResponseSchema>;

export const proxyResponseSchema = z.object({ "admin": z.string(), "caddy": caddyResponseSchema, "enabled": z.boolean(), "in_sync": z.boolean(), "last_applied_at": z.string().nullable(), "last_error": z.string().nullable(), "reachable": z.boolean(), "routes": z.array(routeResponseSchema), "settings": settingsResponseSchema });

export type ProxyResponse = z.infer<typeof proxyResponseSchema>;

export const schema = proxyResponseSchema;
