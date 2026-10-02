import type { ComponentProps } from "react";

import { cn } from "@/shared/lib/cn";

import { FIELD_FOCUS, FIELD_INVALID, FIELD_STYLE } from "./field-style";

export function NativeSelect({ className, ...props }: ComponentProps<"select">) {
  return <select data-slot="native-select" className={cn(FIELD_STYLE, FIELD_FOCUS, FIELD_INVALID, "h-9 pr-8", className)} {...props} />;
}
