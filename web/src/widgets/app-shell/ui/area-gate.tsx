"use client";

import { usePathname } from "next/navigation";
import type { ReactNode } from "react";

import { RequireRight, mayOpen, useSession } from "@/entities/session";

import { areasOfPage } from "../model/page-areas";

export function AreaGate({ children }: { children: ReactNode }) {
  const areas = areasOfPage(usePathname());
  const session = useSession();
  const area = areas.find((candidate) => mayOpen(session.data, candidate)) ?? areas[0];
  return area === undefined ? <>{children}</> : <RequireRight area={area}>{children}</RequireRight>;
}
