"use client";

import { usePathname } from "next/navigation";
import type { ReactNode } from "react";

import { Boundary } from "./boundary";

export function PageTransition({ children }: { children: ReactNode }) {
  const pathname = usePathname();
  return (
    <Boundary key={pathname} name="page" share="page-fade" enter="page-fade" exit="page-fade" default="none">
      <div data-slot="page-transition">{children}</div>
    </Boundary>
  );
}
