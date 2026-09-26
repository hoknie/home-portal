export const routes = {
  home: "/",
  login: "/login/",
  service: (id: string) => `/service/?id=${encodeURIComponent(id)}`,
  adminServices: "/admin/services/",
  adminLayout: "/admin/layout/",
  adminNetwork: "/admin/network/",
  adminProxy: "/admin/proxy/",
  adminAutomations: "/admin/automations/",
  newAutomation: "/admin/automations/new/",
  editAutomation: (id: string) => `/admin/automations/edit/?id=${encodeURIComponent(id)}`,
  adminWebhooks: "/admin/webhooks/",
  newWebhook: "/admin/webhooks/new/",
  editWebhook: (id: string) => `/admin/webhooks/edit/?id=${encodeURIComponent(id)}`,
  webhookDetails: (id: string) => `/admin/webhooks/details/?id=${encodeURIComponent(id)}`,
  newService: "/admin/services/new/",
  editService: (id: string) => `/admin/services/edit/?id=${encodeURIComponent(id)}`,
} as const;

export const api = {
  session: "/api/session",
  services: "/api/services",
  service: (id: string) => `/api/services/${encodeURIComponent(id)}`,
  serviceProbe: (id: string) => `/api/services/${encodeURIComponent(id)}/probe`,
  serviceHistory: (id: string, range: string) =>
    `/api/services/${encodeURIComponent(id)}/history?range=${encodeURIComponent(range)}`,
  network: "/api/network",
  restartPortal: "/api/portal/restart",
  health: "/health",
  dashboard: "/api/dashboard",
  layout: "/api/dashboard?all=true",
  environment: "/api/environment",
  widget: (id: string) => `/api/widgets/${encodeURIComponent(id)}/data`,
  icon: (id: string) => `/api/icons/${encodeURIComponent(id)}`,
  iconPreview: "/api/icon-preview",
  publicPortal: "/api/public/portal",
  publicWidget: (id: string) => `/api/public/widgets/${encodeURIComponent(id)}/data`,
  publicIcon: (id: string) => `/api/public/icons/${encodeURIComponent(id)}`,
  proxy: "/api/proxy",
  dns: "/api/dns",
  proxyApply: "/api/proxy/apply",
  proxyContinue: (to: string) => `/api/proxy/continue?to=${encodeURIComponent(to)}`,
  proxyRootCertificate: "/api/proxy/root-certificate",
  caddySource: "/api/proxy/caddy",
  caddyDownload: "/api/proxy/caddy/download",
  caddyStart: "/api/proxy/caddy/start",
  caddyStop: "/api/proxy/caddy/stop",
  automations: "/api/automations",
  automation: (id: string) => `/api/automations/${encodeURIComponent(id)}`,
  automationRun: (id: string) => `/api/automations/${encodeURIComponent(id)}/run`,
  automationRuns: (filter: { automation?: string | null; webhook?: string | null; text?: string | null }) => {
    const query = new URLSearchParams();
    for (const [name, value] of Object.entries(filter)) {
      if (value) {
        query.set(name, value);
      }
    }
    const text = query.toString();
    return text === "" ? "/api/automations/runs" : `/api/automations/runs?${text}`;
  },
  automationRunItem: (id: string) => `/api/automations/runs/${encodeURIComponent(id)}`,
  automationRunStop: (id: string) => `/api/automations/runs/${encodeURIComponent(id)}/stop`,
  automationCatalogue: "/api/automations/catalogue",
  automationScripts: "/api/automations/scripts",
  automationSchedule: (cron: string) => `/api/automations/schedule?cron=${encodeURIComponent(cron)}`,
  webhooks: "/api/webhooks",
  webhook: (id: string) => `/api/webhooks/${encodeURIComponent(id)}`,
  webhookToken: (id: string) => `/api/webhooks/${encodeURIComponent(id)}/token`,
} as const;

export const STATUS_REFRESH_MILLISECONDS = 10_000;
