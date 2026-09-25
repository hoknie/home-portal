import { api } from "@/shared/config";

export const LUCIDE_PREFIX = "lucide:";

export const FETCHED = ["auto", "file:", "url:", "catalog:"];

export type ServiceIconChoice = { name: string | null; source: string | null };

export function iconOf(service: { id: string; icon: string | null }, scope: "private" | "public" = "private"): ServiceIconChoice {
  const icon = service.icon;
  if (!icon) {
    return { name: null, source: null };
  }
  if (icon.startsWith("/")) {
    return { name: null, source: icon };
  }
  if (icon.startsWith(LUCIDE_PREFIX)) {
    return { name: icon.slice(LUCIDE_PREFIX.length), source: null };
  }
  if (FETCHED.some((prefix) => icon === prefix || icon.startsWith(prefix))) {
    return { name: null, source: scope === "public" ? api.publicIcon(service.id) : api.icon(service.id) };
  }
  return { name: icon, source: null };
}
