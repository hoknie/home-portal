"use client";

import { usePathname } from "next/navigation";
import { useSyncExternalStore } from "react";

export const ADDRESS_EVENT = "portal:address";
export const ADDRESS_LINK_ATTRIBUTE = "data-address-link";

type Guard = () => boolean;

const guards = new Set<Guard>();
let current: string | null = null;

function here() {
  return `${window.location.pathname}${window.location.search}`;
}

export function mayLeave() {
  return [...guards].every((guard) => guard());
}

export function guardLeaving(guard: Guard) {
  guards.add(guard);
  return () => {
    guards.delete(guard);
  };
}

function moved(href: string, change: (url: string) => void) {
  if (href === here()) {
    return true;
  }
  if (!mayLeave()) {
    return false;
  }
  change(href);
  current = here();
  window.dispatchEvent(new Event(ADDRESS_EVENT));
  return true;
}

export function pushAddress(href: string) {
  return moved(href, (url) => window.history.pushState(null, "", url));
}

export function replaceAddress(href: string) {
  return moved(href, (url) => window.history.replaceState(null, "", url));
}

function popped() {
  const left = current;
  if (left !== null && left !== here() && !mayLeave()) {
    window.history.pushState(null, "", left);
    return;
  }
  current = here();
  window.dispatchEvent(new Event(ADDRESS_EVENT));
}

function subscribe(notify: () => void) {
  current = here();
  window.addEventListener("popstate", popped);
  window.addEventListener(ADDRESS_EVENT, notify);
  return () => {
    window.removeEventListener("popstate", popped);
    window.removeEventListener(ADDRESS_EVENT, notify);
  };
}

export function useAddress() {
  usePathname();
  const path = useSyncExternalStore(
    subscribe,
    () => window.location.pathname,
    () => null,
  );
  return { path, push: pushAddress, replace: replaceAddress };
}
