import type { ReactNode } from "react";

import { cn } from "@/shared/lib/cn";

export type TextSize = "caption" | "body" | "lead";

export type TextTone = "default" | "muted" | "accent" | "danger";

const SIZE: Record<TextSize, string> = { caption: "text-xs", body: "text-sm", lead: "text-base" };

const TONE: Record<TextTone, string> = {
  default: "text-foreground",
  muted: "text-muted-foreground",
  accent: "text-primary",
  danger: "text-destructive",
};

export type TextProps = {
  as?: "p" | "span" | "div";
  size?: TextSize;
  tone?: TextTone;
  id?: string;
  role?: "alert" | "status";
  className?: string;
  children: ReactNode;
};

export function Text({ as: Element = "p", size = "body", tone = "default", id, role, className, children }: TextProps) {
  return (
    <Element id={id} role={role} data-slot="text" className={cn(SIZE[size], TONE[tone], className)}>
      {children}
    </Element>
  );
}
