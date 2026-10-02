"use client";

import { SearchX } from "lucide-react";
import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import { routes } from "@/shared/config";
import type { TrailItem } from "@/shared/lib/breadcrumbs";
import { AddressLink } from "@/shared/ui/address-link";
import { Appear, buttonVariants, EmptyState, ErrorNotice, Skeleton, SkeletonCard } from "@/shared/ui/kit";
import { PageHeader } from "@/shared/ui/page-header";

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
        {data.failure ? <ErrorNotice title={t("errors.loadFailed")} description={data.failure.message} onRetry={data.retry} /> : (
          <div className="grid gap-4 lg:grid-cols-[1fr_20rem]" data-skeleton="workflow-editor" aria-busy="true">
            <Skeleton className="h-[60svh] w-full rounded-2xl" />
            <SkeletonCard lines={6} />
          </div>
        )}
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
      <Appear className="grid gap-4">{children}</Appear>
    </div>
  );
}
