import type { ComponentProps } from "react";

import { cn } from "@/shared/lib/cn";

export function Inset({ className, ...props }: ComponentProps<"div">) {
  return <div data-slot="inset" className={cn("surface-inset rounded-lg p-3", className)} {...props} />;
}
