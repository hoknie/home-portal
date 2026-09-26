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

export const LEFT_KEY = "portal_left_to";
export const RETURN_WINDOW_MILLISECONDS = 30_000;

export function hostOf(value: string | null) {
  try {
    return value ? new URL(value).host : "";
  } catch {
    return "";
  }
}

export function cameBackFrom(href: string, now = Date.now()) {
  try {
    const left = JSON.parse(window.sessionStorage.getItem(LEFT_KEY) ?? "null") as { href?: string; at?: number } | null;
    return left?.href === href && typeof left.at === "number" && now - left.at < RETURN_WINDOW_MILLISECONDS;
  } catch {
    return false;
  }
}

function rememberLeaving(href: string) {
  try {
    window.sessionStorage.setItem(LEFT_KEY, JSON.stringify({ href, at: Date.now() }));
  } catch {
    return;
  }
}

export function leaveTo(href: string) {
  rememberLeaving(href);
  window.location.assign(href);
}
