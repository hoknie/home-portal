"use client";

import { SearchX } from "lucide-react";
import Link from "next/link";
import { useSearchParams } from "next/navigation";
import { useTranslations } from "next-intl";

import { useEnvironment } from "@/entities/environment";
import { useServices } from "@/entities/service";
import { routes } from "@/shared/config";
import { useTrail } from "@/shared/lib/breadcrumbs";
import { Breadcrumbs } from "@/shared/ui/breadcrumbs";
import { Appear, Button, EmptyState, Loaded, Page, SkeletonCard } from "@/shared/ui/kit";

import { AddressesCard } from "./addresses-card";
import { DetailsCard } from "./details-card";
import { HistoryCard } from "./history-card";
import { ProbeCard } from "./probe-card";
import { RelatedWidgets } from "./related-widgets";
import { ServiceSummary } from "./service-summary";

export const ID_PARAMETER = "id";

function ServicePageSkeleton() {
  return (
    <div className="grid gap-6" data-skeleton="service-page" aria-busy="true">
      <SkeletonCard lines={2} />
      <div className="grid gap-6 lg:grid-cols-2">
        <SkeletonCard lines={3} />
        <SkeletonCard lines={3} />
      </div>
      <SkeletonCard lines={4} />
    </div>
  );
}

export function ServicePageScreen() {
  const t = useTranslations();
  const id = useSearchParams().get(ID_PARAMETER) ?? "";
  const services = useServices();
  const environment = useEnvironment();
  const trail = useTrail();
  const named = services.data?.data.services.find((candidate) => candidate.id === id)?.name ?? id;
  const crumbs = <Breadcrumbs items={trail.of({ label: named })} />;
  if (!services.data) {
    return (
      <Page>
        {crumbs}
        <Loaded query={services} skeleton={<ServicePageSkeleton />}>
          {() => null}
        </Loaded>
      </Page>
    );
  }
  const all = services.data.data.services;
  const service = all.find((candidate) => candidate.id === id);
  if (!service) {
    return (
      <Page>
        {crumbs}
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
      </Page>
    );
  }
  return (
    <Page>
      {crumbs}
      <Appear className="grid gap-6">
        <ServiceSummary service={service} revision={services.data.revision} />
        <div className="grid gap-6 lg:grid-cols-2">
          <ProbeCard service={service} />
          <AddressesCard service={service} environment={environment.data?.environment ?? null} />
        </div>
        <HistoryCard id={service.id} />
        <DetailsCard service={service} />
        <RelatedWidgets service={service} services={all} />
      </Appear>
    </Page>
  );
}
