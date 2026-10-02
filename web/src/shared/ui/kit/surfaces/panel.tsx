import type { ComponentProps } from "react";

import { cn } from "@/shared/lib/cn";

export type PanelProps = ComponentProps<"section"> & {
  as?: "section" | "div" | "aside" | "header" | "nav" | "footer";
  padding?: "none" | "compact" | "default";
};

const PADDING = { none: "", compact: "p-3", default: "p-4 sm:p-5" } as const;

export function Panel({ as: Element = "section", className, padding = "default", ...props }: PanelProps) {
  return <Element data-slot="panel" className={cn("surface-panel rounded-2xl", PADDING[padding], className)} {...(props as ComponentProps<"div">)} />;
}
