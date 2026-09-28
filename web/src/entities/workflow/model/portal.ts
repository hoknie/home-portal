import { z } from "zod";

export const portalServiceSchema = z.object({
  id: z.string(),
  name: z.string(),
  group: z.string().nullable(),
  url: z.string(),
  address: z.string(),
  state: z.string(),
  since: z.string().nullable(),
  latency_milliseconds: z.number().nullable(),
  public: z.boolean(),
});

export const portalValuesSchema = z.object({
  services: z.array(portalServiceSchema),
  network: z.object({ address: z.string(), port: z.number(), url: z.string() }),
  modules: z.record(z.string(), z.object({ is_enabled: z.boolean() })),
  environments: z.array(z.string()),
});

export type PortalValues = z.infer<typeof portalValuesSchema>;

export const SERVICE_FIELDS = ["id", "name", "group", "url", "address", "state", "since", "latency_milliseconds", "public"] as const;

export const NETWORK_FIELDS = ["address", "port", "url"] as const;

export function portalProblem(name: string, portal: PortalValues | null | undefined): boolean {
  const parts = name.split(".").slice(1);
  const [area, key, field] = parts;
  if (parts.length === 1) {
    return !["services", "environments", "network", "modules"].includes(area);
  }
  switch (area) {
    case "services":
      if (portal && !portal.services.some((service) => service.id === key)) {
        return true;
      }
      return field !== undefined && !(SERVICE_FIELDS as readonly string[]).includes(field);
    case "network":
      return parts.length !== 2 || !(NETWORK_FIELDS as readonly string[]).includes(key);
    case "modules":
      if (portal && !(key in portal.modules)) {
        return true;
      }
      return field !== undefined && field !== "is_enabled";
    default:
      return true;
  }
}

export function portalValue(name: string, portal: PortalValues | null | undefined): unknown {
  if (!portal) {
    return undefined;
  }
  const [area, key, ...rest] = name.split(".").slice(1);
  if (area === "services" && key !== undefined) {
    const service = portal.services.find((entry) => entry.id === key);
    return rest.reduce<unknown>((value, part) => (value && typeof value === "object" ? (value as Record<string, unknown>)[part] : undefined), service);
  }
  return [key, ...rest].filter((part) => part !== undefined).reduce<unknown>(
    (value, part) => (value && typeof value === "object" ? (value as Record<string, unknown>)[part] : undefined),
    (portal as Record<string, unknown>)[area],
  );
}

function example(value: unknown): string {
  return value === null || value === undefined ? "" : typeof value === "string" ? value : JSON.stringify(value);
}

export type PortalSuggestion = { value: string; kind: "services" | "service" | "network" | "module" | "environments"; subject: string; example: string };

export function portalSuggestions(portal: PortalValues | null | undefined): PortalSuggestion[] {
  if (!portal) {
    return [];
  }
  return [
    { value: "portal.services", kind: "services", subject: "", example: String(portal.services.length) },
    ...portal.services.flatMap((service) =>
      SERVICE_FIELDS.map<PortalSuggestion>((field) => ({
        value: `portal.services.${service.id}.${field}`,
        kind: "service",
        subject: `${service.name} · ${field}`,
        example: example(service[field]),
      })),
    ),
    ...NETWORK_FIELDS.map<PortalSuggestion>((field) => ({ value: `portal.network.${field}`, kind: "network", subject: field, example: example(portal.network[field]) })),
    ...Object.entries(portal.modules).map<PortalSuggestion>(([name, module]) => ({
      value: `portal.modules.${name}.is_enabled`,
      kind: "module",
      subject: name,
      example: String(module.is_enabled),
    })),
    { value: "portal.environments", kind: "environments", subject: "", example: portal.environments.join(", ") },
  ];
}
