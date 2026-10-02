import type { ComponentProps } from "react";

import { cn } from "@/shared/lib/cn";

export function Radio({ className, ...props }: Omit<ComponentProps<"input">, "type">) {
  return <input type="radio" data-slot="radio" className={cn("size-4 shrink-0 accent-primary disabled:cursor-not-allowed disabled:opacity-50", className)} {...props} />;
}
