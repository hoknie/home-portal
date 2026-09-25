import { request } from "@/shared/api";
import { api } from "@/shared/config";

import { type CaddySourceForm, type ProxySettingsForm, proxySettingsRequestOf } from "../model/form";
import { proxySchema } from "../model/schema";

export function fetchProxy() {
  return request(api.proxy, { schema: proxySchema });
}

export function applyProxy() {
  return request(api.proxyApply, { method: "POST", schema: proxySchema });
}

export function downloadCaddy() {
  return request(api.caddyDownload, { method: "POST", schema: proxySchema });
}

export function startCaddy(revision: string | null) {
  return request(api.caddyStart, { method: "POST", revision, schema: proxySchema });
}

export function stopCaddy(revision: string | null) {
  return request(api.caddyStop, { method: "POST", revision, schema: proxySchema });
}

export function saveProxySettings(form: ProxySettingsForm, revision: string | null) {
  return request(api.proxy, { method: "PUT", body: proxySettingsRequestOf(form), revision, schema: proxySchema });
}

export function saveCaddySource(form: CaddySourceForm, revision: string | null) {
  return request(api.caddySource, { method: "PUT", body: form, revision, schema: proxySchema });
}
