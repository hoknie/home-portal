import { routes } from "@/shared/config";

export function safeNext(value: string | null): string {
  if (!value || !value.startsWith("/") || value.startsWith("//") || value.startsWith(routes.login)) {
    return routes.home;
  }
  return value;
}
