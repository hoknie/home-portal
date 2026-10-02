import type { ReactNode } from "react";

import { cn } from "@/shared/lib/cn";

export type AppearProps = { className?: string; children: ReactNode };

export function Appear({ className, children }: AppearProps) {
  return (
    <div data-slot="appear" className={cn("animate-in fade-in-0 duration-150", className)}>
      {children}
    </div>
  );
}
