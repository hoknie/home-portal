"use client";

import { SearchX } from "lucide-react";
import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import { routes } from "@/shared/config";
import type { TrailItem } from "@/shared/lib/breadcrumbs";
import { AddressLink } from "@/shared/ui/address-link";
import { EmptyState } from "@/shared/ui/empty-state";
import { ErrorNotice } from "@/shared/ui/error-notice";
import { PageHeader } from "@/shared/ui/page-header";
import { Skeleton, buttonVariants } from "@/shared/ui/primitives";

export type WorkflowFrameProps = {
  crumbs: TrailItem[];
  title: string;
  description: string;
  data: { ready: boolean; failure: Error | null; retry: () => void };
  missing: boolean;
  children: ReactNode;
};

export function WorkflowFrame({ crumbs, title, description, data, missing, children }: WorkflowFrameProps) {
  const t = useTranslations();
  const header = <PageHeader breadcrumbs={crumbs} title={title} description={description} />;
  if (!data.ready) {
    return (
      <div className="grid gap-6">
        {header}
        {data.failure ? <ErrorNotice title={t("errors.loadFailed")} description={data.failure.message} onRetry={data.retry} /> : <Skeleton className="h-96 w-full" aria-busy="true" />}
      </div>
    );
  }
  if (missing) {
    return (
      <div className="grid gap-6">
        {header}
        <EmptyState
          icon={SearchX}
          title={t("workflowEditor.notFound")}
          description={t("workflowEditor.notFoundHint")}
          action={
            <AddressLink href={routes.adminWorkflows} className={buttonVariants({ variant: "outline" })}>
              {t("workflowEditor.backToWorkflows")}
            </AddressLink>
          }
        />
      </div>
    );
  }
  return (
    <div className="grid gap-4">
      {header}
      {children}
    </div>
  );
}
