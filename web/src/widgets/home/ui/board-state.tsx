"use client";

import { useTranslations } from "next-intl";

import { ErrorNotice } from "@/shared/ui/error-notice";
import { Skeleton } from "@/shared/ui/primitives";

export type BoardStateProps = { error: Error | null; onRetry: () => void };

export function BoardState({ error, onRetry }: BoardStateProps) {
  const t = useTranslations("errors");
  if (error) {
    return <ErrorNotice title={t("loadFailed")} description={error.message} onRetry={onRetry} />;
  }
  return (
    <div className="grid gap-4" aria-busy="true">
      <Skeleton className="h-24 w-full" />
      <Skeleton className="h-48 w-full" />
    </div>
  );
}
