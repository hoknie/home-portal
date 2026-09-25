import type { ServiceView } from "@/entities/service";

export type ServiceGroup = { name: string | null; services: ServiceView[] };

export function groupServices(services: ServiceView[], groups?: string[]): ServiceGroup[] {
  if (groups) {
    return groups.map((name) => ({ name, services: services.filter((service) => service.group === name) }));
  }
  const named = new Map<string, ServiceView[]>();
  const ungrouped: ServiceView[] = [];
  for (const service of services) {
    if (service.group) {
      named.set(service.group, [...(named.get(service.group) ?? []), service]);
    } else {
      ungrouped.push(service);
    }
  }
  const result: ServiceGroup[] = [...named].map(([name, members]) => ({ name, services: members }));
  if (ungrouped.length > 0) {
    result.push({ name: null, services: ungrouped });
  }
  return result;
}

export function countByState(services: ServiceView[]) {
  const counts = { up: 0, degraded: 0, down: 0, unreadable: 0, unknown: 0 };
  for (const service of services) {
    counts[service.status?.state ?? "unknown"] += 1;
  }
  return counts;
}
