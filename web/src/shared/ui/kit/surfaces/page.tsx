import type { ComponentProps } from "react";

import { cn } from "@/shared/lib/cn";

export function Page({ className, ...props }: ComponentProps<"div">) {
  return <div data-slot="page" className={cn("grid content-start gap-6", className)} {...props} />;
}
