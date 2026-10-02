import type { ComponentProps } from "react";

import { cn } from "@/shared/lib/cn";

export function BareButton({ className, type = "button", ...props }: ComponentProps<"button">) {
  return (
    <button
      data-slot="bare-button"
      type={type}
      className={cn("outline-none focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:pointer-events-none disabled:opacity-50", className)}
      {...props}
    />
  );
}
