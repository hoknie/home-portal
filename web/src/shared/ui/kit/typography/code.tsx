import type { ReactNode } from "react";

import { cn } from "@/shared/lib/cn";

export type CodeProps = { className?: string; children: ReactNode };

export function Code({ className, children }: CodeProps) {
  return <code data-slot="code" className={cn("rounded-md bg-glass-tint px-1.5 py-0.5 font-mono text-[0.85em] break-all", className)}>{children}</code>;
}
