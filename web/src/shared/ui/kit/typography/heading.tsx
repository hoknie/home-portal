import type { ReactNode } from "react";

import { cn } from "@/shared/lib/cn";

export type HeadingLevel = "page" | "section" | "group";

const ELEMENT = { page: "h1", section: "h2", group: "h3" } as const;

const STYLE: Record<HeadingLevel, string> = {
  page: "text-2xl font-semibold tracking-tight",
  section: "text-lg font-semibold",
  group: "text-sm font-semibold",
};

export type HeadingProps = { level: HeadingLevel; as?: "h1" | "h2" | "h3" | "h4"; id?: string; className?: string; children: ReactNode };

export function Heading({ level, as, id, className, children }: HeadingProps) {
  const Element = as ?? ELEMENT[level];
  return (
    <Element id={id} data-slot="heading" data-level={level} className={cn(STYLE[level], className)}>
      {children}
    </Element>
  );
}
