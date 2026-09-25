import { api, routes } from "@/shared/config";

import { safeNext } from "./safe-next";

export const RETURN_PARAMETER = "return";

export type Destination = { href: string; leavesTheInterface: boolean };

function absolute(value: string) {
  try {
    const url = new URL(value);
    return url.protocol === "https:" || url.protocol === "http:";
  } catch {
    return false;
  }
}

export function destinationOf(next: string | null, returning: string | null): Destination {
  if (returning) {
    if (absolute(returning)) {
      return { href: api.proxyContinue(returning), leavesTheInterface: true };
    }
    const path = safeNext(returning);
    if (path !== routes.home || returning === routes.home) {
      return { href: path, leavesTheInterface: false };
    }
  }
  return { href: safeNext(next), leavesTheInterface: false };
}

export function leaveTo(href: string) {
  window.location.assign(href);
}
