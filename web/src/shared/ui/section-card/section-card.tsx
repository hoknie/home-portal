import type { ReactNode } from "react";

import { cn } from "@/shared/lib/cn";
import { Card, CardAction, CardContent, CardDescription, CardHeader, CardTitle } from "@/shared/ui/kit";

export type SectionCardProps = {
  title?: string;
  description?: string;
  badge?: ReactNode;
  actions?: ReactNode;
  flush?: boolean;
  hoverable?: boolean;
  children: ReactNode;
};

export function SectionCard({ title, description, badge, actions, flush = false, hoverable = false, children }: SectionCardProps) {
  const hasHeader = Boolean(title || description || badge || actions);
  return (
    <Card className={cn("gap-4", flush && "overflow-hidden pb-0", flush && !hasHeader && "gap-0 py-0", hoverable && "transition-colors hover:border-primary/40")}>
      {hasHeader ? (
        <CardHeader>
          <div className="flex items-center gap-2">
            {title ? <CardTitle>{title}</CardTitle> : null}
            {badge}
          </div>
          {description ? <CardDescription>{description}</CardDescription> : null}
          {actions ? <CardAction className="flex items-center gap-2">{actions}</CardAction> : null}
        </CardHeader>
      ) : null}
      <CardContent className={cn(flush && "px-0")}>{children}</CardContent>
    </Card>
  );
}
