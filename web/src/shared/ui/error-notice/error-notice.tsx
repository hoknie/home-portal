"use client";

import { AlertTriangle } from "lucide-react";
import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import { Button } from "@/shared/ui/primitives";

export type ErrorNoticeProps = {
  title: string;
  description?: string;
  onRetry?: () => void;
  action?: ReactNode;
};

export function ErrorNotice({ title, description, onRetry, action }: ErrorNoticeProps) {
  const t = useTranslations("common");
  return (
    <div role="alert" className="flex items-start gap-3 rounded-xl border border-destructive/30 bg-destructive/5 p-4">
      <AlertTriangle className="mt-0.5 size-5 shrink-0 text-destructive" aria-hidden />
      <div className="flex-1 space-y-1">
        <p className="text-sm font-medium">{title}</p>
        {description ? <p className="text-sm whitespace-pre-line text-muted-foreground">{description}</p> : null}
        {action ? <div className="pt-2">{action}</div> : null}
      </div>
      {onRetry ? (
        <Button size="sm" variant="outline" onClick={onRetry}>
          {t("retry")}
        </Button>
      ) : null}
    </div>
  );
}
