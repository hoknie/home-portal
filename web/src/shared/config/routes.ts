export const routes = {
  home: "/",
  login: "/login/",
  service: (id: string) => `/service/?id=${encodeURIComponent(id)}`,
  adminServices: "/admin/services/",
  adminLayout: "/admin/layout/",
  adminNetwork: "/admin/network/",
  adminModules: "/admin/modules/",
  adminProxy: "/admin/proxy/",
  adminDns: "/admin/dns/",
  adminAutomations: "/admin/automations/",
  newAutomation: "/admin/automations/new/",
  editAutomation: (id: string) => `/admin/automations/edit/?id=${encodeURIComponent(id)}`,
  adminWebhooks: "/admin/webhooks/",
  adminUsers: "/admin/users/",
  adminWorkflows: "/admin/workflows/",
  newWorkflow: "/admin/workflows/new/",
  editWorkflow: (id: string) => `/admin/workflows/edit/?id=${encodeURIComponent(id)}`,
  adminNotifications: "/admin/notifications/",
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
  modules: "/api/modules",
  module: (name: string) => `/api/modules/${encodeURIComponent(name)}`,
  users: "/api/users",
  user: (name: string) => `/api/users/${encodeURIComponent(name)}`,
  userPassword: (name: string) => `/api/users/${encodeURIComponent(name)}/password`,
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
  automationRuns: (filter: { automation?: string | null; webhook?: string | null; workflow?: string | null; text?: string | null }) => {
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
  workflows: "/api/workflows",
  workflow: (id: string) => `/api/workflows/${encodeURIComponent(id)}`,
  workflowRun: (id: string) => `/api/workflows/${encodeURIComponent(id)}/run`,
  workflowCatalogue: "/api/workflows/catalogue",
  workflowPortal: "/api/workflows/portal",
  secrets: "/api/secrets",
  notifications: "/api/notifications",
  notificationChannel: (name: string) => `/api/notifications/channels/${encodeURIComponent(name)}`,
  notificationTest: "/api/notifications/test",
} as const;

export const MODULE_PAGES = {
  proxy: routes.adminProxy,
  dns: routes.adminDns,
  automations: routes.adminAutomations,
  webhooks: routes.adminWebhooks,
  users: routes.adminUsers,
  workflows: routes.adminWorkflows,
  notifications: routes.adminNotifications,
} as const;

export const STATUS_REFRESH_MILLISECONDS = 10_000;
