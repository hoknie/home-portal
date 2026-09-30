import { routes } from "@/shared/config";

const PORTAL = "http://portal.invalid";
const UNSAFE = /[\\\u0000-\u001f\u007f]/;

export function safeNext(value: string | null): string {
  if (!value || !value.startsWith("/") || value.startsWith("//") || UNSAFE.test(value)) {
    return routes.home;
  }
  let url: URL;
  try {
    url = new URL(value, PORTAL);
  } catch {
    return routes.home;
  }
  const path = `${url.pathname}${url.search}${url.hash}`;
  if (url.origin !== PORTAL || path.startsWith(routes.login)) {
    return routes.home;
  }
  return path;
}
