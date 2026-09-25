"use client";

import { SearchX } from "lucide-react";
import Link from "next/link";
import { useRouter, useSearchParams } from "next/navigation";
import { useLocale, useTranslations } from "next-intl";

import { ServiceForm } from "@/features/service-form";
import { groupsOf, useServices } from "@/entities/service";
import { routes } from "@/shared/config";
import { EmptyState } from "@/shared/ui/empty-state";
import { ErrorNotice } from "@/shared/ui/error-notice";
import { PageHeader } from "@/shared/ui/page-header";
import { Button, Skeleton } from "@/shared/ui/primitives";

export const ID_PARAMETER = "id";

export type ServiceEditorScreenProps = { mode: "new" | "edit" };

export function ServiceEditorScreen({ mode }: ServiceEditorScreenProps) {
  const t = useTranslations();
  const locale = useLocale();
  const router = useRouter();
  const id = useSearchParams().get(ID_PARAMETER) ?? "";
  const services = useServices();
  const title = t(mode === "new" ? "serviceForm.addTitle" : "serviceForm.editTitle");
  const header = <PageHeader title={title} description={t("serviceForm.description")} />;
  if (!services.data) {
    return (
      <div className="grid gap-8">
        {header}
        {services.error ? (
          <ErrorNotice title={t("errors.loadFailed")} description={services.error.message} onRetry={() => void services.refetch()} />
        ) : (
          <Skeleton className="h-96 w-full" aria-busy="true" />
        )}
      </div>
    );
  }
  const all = services.data.data.services;
  const service = mode === "edit" ? (all.find((candidate) => candidate.id === id) ?? null) : null;
  if (mode === "edit" && !service) {
    return (
      <div className="grid gap-8">
        {header}
        <EmptyState
          icon={SearchX}
          title={t("serviceForm.notFound")}
          description={t("serviceForm.notFoundHint")}
          action={
            <Button asChild variant="outline">
              <Link href={routes.adminServices}>{t("serviceForm.backToServices")}</Link>
            </Button>
          }
        />
      </div>
    );
  }
  return (
    <div className="grid gap-8">
      {header}
      <ServiceForm
        key={service?.id ?? "new"}
        service={service}
        revision={services.data.revision}
        taken={all.map((candidate) => candidate.id).filter((candidate) => candidate !== service?.id)}
        groups={groupsOf(all, locale)}
        onSaved={() => router.push(routes.adminServices)}
        onConflict={() => void services.refetch()}
      />
    </div>
  );
}
