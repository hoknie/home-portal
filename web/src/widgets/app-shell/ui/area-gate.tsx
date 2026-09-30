"use client";

import { usePathname } from "next/navigation";
import type { ReactNode } from "react";

import { RequireRight } from "@/entities/session";

import { areaOfPage } from "../model/page-areas";

export function AreaGate({ children }: { children: ReactNode }) {
  const area = areaOfPage(usePathname());
  return area === null ? <>{children}</> : <RequireRight area={area}>{children}</RequireRight>;
}
