"use client";

import type { ReactNode } from "react";

import { useCan } from "../model/can";
import type { Action, Area } from "../model/rights";

export type AllowedProps = { area: Area; action: Action; children: ReactNode };

export function Allowed({ area, action, children }: AllowedProps) {
  const can = useCan();
  return can(area, action) ? <>{children}</> : null;
}
