"use client";

import type { ReactNode } from "react";

import { NoAccess } from "@/shared/ui/no-access";

import { useMayOpen } from "../model/can";
import type { Area } from "../model/rights";

export type RequireRightProps = { area: Area; children: ReactNode };

export function RequireRight({ area, children }: RequireRightProps) {
  const { ready, allowed } = useMayOpen(area);
  if (!ready) {
    return null;
  }
  return allowed ? <>{children}</> : <NoAccess />;
}
