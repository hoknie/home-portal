"use client";

import { Plus, Server } from "lucide-react";
import Link from "next/link";
import { useLocale, useTranslations } from "next-intl";

import { Allowed } from "@/entities/session";
import { useServices } from "@/entities/service";
import { routes } from "@/shared/config";
import { DataTable } from "@/shared/ui/data-table";
import { EmptyState } from "@/shared/ui/empty-state";
import { ErrorNotice } from "@/shared/ui/error-notice";
import { useTrail } from "@/shared/lib/breadcrumbs";
import { PageHeader } from "@/shared/ui/page-header";
import { Button, Skeleton } from "@/shared/ui/primitives";
import { SectionCard } from "@/shared/ui/section-card";

import { byGroup } from "../model/groups";
import { useServiceColumns } from "./service-columns";

export function ServicesScreen() {
  const trail = useTrail();
  const t = useTranslations();
  const locale = useLocale();
  const services = useServices();
  const revision = services.data?.revision ?? null;
  const columns = useServiceColumns(revision);
  const add = (
    <Allowed area="services" action="create">
      <Button asChild>
        <Link href={routes.newService}>
          <Plus aria-hidden />
          {t("services.add")}
        </Link>
      </Button>
    </Allowed>
  );
  return (
    <div className="grid gap-8">
      <PageHeader breadcrumbs={trail.of(trail.section("services"))} title={t("services.title")} description={t("services.subtitle")} actions={add} />
      {services.error && !services.data ? (
        <ErrorNotice title={t("errors.loadFailed")} description={services.error.message} onRetry={() => void services.refetch()} />
      ) : null}
      {services.data && services.data.data.services.length === 0 ? (
        <SectionCard flush>
          <EmptyState icon={Server} title={t("services.empty")} description={t("services.emptyHint")} action={add} />
        </SectionCard>
      ) : services.data ? (
        byGroup(services.data.data.services, locale).map((group) => (
          <SectionCard key={group.name ?? ""} title={group.name ?? t("services.noGroup")} flush>
            <DataTable columns={columns} rows={group.items} rowKey={(service) => service.id} />
          </SectionCard>
        ))
      ) : services.error ? null : (
        <Skeleton className="h-64 w-full" aria-busy="true" />
      )}
    </div>
  );
}
