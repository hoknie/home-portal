"use client";

import { SearchX } from "lucide-react";
import Link from "next/link";
import { useSearchParams } from "next/navigation";
import { useTranslations } from "next-intl";

import { useEnvironment } from "@/entities/environment";
import { useServices } from "@/entities/service";
import { routes } from "@/shared/config";
import { EmptyState } from "@/shared/ui/empty-state";
import { ErrorNotice } from "@/shared/ui/error-notice";
import { Button, Skeleton } from "@/shared/ui/primitives";

import { AddressesCard } from "./addresses-card";
import { DetailsCard } from "./details-card";
import { HistoryCard } from "./history-card";
import { ProbeCard } from "./probe-card";
import { RelatedWidgets } from "./related-widgets";
import { ServiceSummary } from "./service-summary";

export const ID_PARAMETER = "id";

export function ServicePageScreen() {
  const t = useTranslations();
  const id = useSearchParams().get(ID_PARAMETER) ?? "";
  const services = useServices();
  const environment = useEnvironment();
  if (!services.data) {
    return services.error ? (
      <ErrorNotice title={t("errors.loadFailed")} description={services.error.message} onRetry={() => void services.refetch()} />
    ) : (
      <Skeleton className="h-64 w-full" aria-busy="true" />
    );
  }
  const all = services.data.data.services;
  const service = all.find((candidate) => candidate.id === id);
  if (!service) {
    return (
      <EmptyState
        icon={SearchX}
        title={t("servicePage.notFound")}
        description={t("servicePage.notFoundHint")}
        action={
          <Button asChild variant="outline">
            <Link href={routes.home}>{t("servicePage.backHome")}</Link>
          </Button>
        }
      />
    );
  }
  return (
    <div className="grid gap-6">
      <ServiceSummary service={service} />
      <div className="grid gap-6 lg:grid-cols-2">
        <ProbeCard service={service} />
        <AddressesCard service={service} environment={environment.data?.environment ?? null} />
      </div>
      <HistoryCard id={service.id} />
      <DetailsCard service={service} />
      <RelatedWidgets service={service} services={all} />
    </div>
  );
}
