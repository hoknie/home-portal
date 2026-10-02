import type { ComponentProps } from "react";

import { cn } from "@/shared/lib/cn";

import { FIELD_FOCUS, FIELD_INVALID, FIELD_STYLE } from "./field-style";

export function Textarea({ className, ...props }: ComponentProps<"textarea">) {
  return <textarea data-slot="textarea" className={cn(FIELD_STYLE, FIELD_FOCUS, FIELD_INVALID, "min-h-20 py-2", className)} {...props} />;
}
