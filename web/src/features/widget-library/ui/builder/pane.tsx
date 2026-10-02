"use client";

import type { CSSProperties, ReactNode } from "react";

import { cn } from "@/shared/lib/cn";
import { Heading, Panel } from "@/shared/ui/kit";

export type PaneProps = { id: string; title: string; hint?: string; icon?: ReactNode; actions?: ReactNode; className?: string; bodyClassName?: string; style?: CSSProperties; children: ReactNode };

export function Pane({ id, title, hint, icon, actions, className, bodyClassName, style, children }: PaneProps) {
  return (
    <Panel padding="none" aria-labelledby={id} style={style} className={cn("flex min-h-0 min-w-0 flex-col overflow-hidden rounded-2xl", className)}>
      <header className="flex min-h-12 items-center gap-2 border-b border-glass-edge px-4 py-2">
        {icon}
        <div className="grid min-w-0 flex-1 gap-0.5">
          <Heading level="group" as="h2" id={id} className="text-sm font-semibold">
            {title}
          </Heading>
          {hint ? <p className="text-xs text-muted-foreground">{hint}</p> : null}
        </div>
        {actions ? <div className="flex shrink-0 items-center gap-1">{actions}</div> : null}
      </header>
      <div className={cn("min-h-0 flex-1 overflow-y-auto p-4", bodyClassName)}>{children}</div>
    </Panel>
  );
}
