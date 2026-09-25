export const routes = {
  home: "/",
  login: "/login/",
  service: (id: string) => `/service/?id=${encodeURIComponent(id)}`,
  adminServices: "/admin/services/",
  adminLayout: "/admin/layout/",
  adminNetwork: "/admin/network/",
  adminProxy: "/admin/proxy/",
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
  proxyApply: "/api/proxy/apply",
  proxyContinue: (to: string) => `/api/proxy/continue?to=${encodeURIComponent(to)}`,
  proxyRootCertificate: "/api/proxy/root-certificate",
  caddySource: "/api/proxy/caddy",
  caddyDownload: "/api/proxy/caddy/download",
  caddyStart: "/api/proxy/caddy/start",
  caddyStop: "/api/proxy/caddy/stop",
} as const;

export const STATUS_REFRESH_MILLISECONDS = 10_000;
