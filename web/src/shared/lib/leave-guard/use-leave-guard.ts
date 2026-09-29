"use client";

import { useEffect } from "react";

import { ADDRESS_LINK_ATTRIBUTE, guardLeaving } from "@/shared/lib/navigation";

export function useLeaveGuard(dirty: boolean, message: string) {
  useEffect(() => {
    if (!dirty) {
      return;
    }
    const beforeUnload = (event: BeforeUnloadEvent) => {
      event.preventDefault();
    };
    const click = (event: MouseEvent) => {
      const anchor = event.target instanceof Element ? event.target.closest("a[href]") : null;
      if (!anchor || anchor.getAttribute("target") === "_blank" || anchor.hasAttribute(ADDRESS_LINK_ATTRIBUTE)) {
        return;
      }
      if (!window.confirm(message)) {
        event.preventDefault();
        event.stopPropagation();
      }
    };
    const release = guardLeaving(() => window.confirm(message));
    window.addEventListener("beforeunload", beforeUnload);
    document.addEventListener("click", click, true);
    return () => {
      release();
      window.removeEventListener("beforeunload", beforeUnload);
      document.removeEventListener("click", click, true);
    };
  }, [dirty, message]);
}
