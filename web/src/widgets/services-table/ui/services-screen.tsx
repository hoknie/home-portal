"use client";

import { Plus, Server } from "lucide-react";
import Link from "next/link";
import { useRouter, useSearchParams } from "next/navigation";
import { useTranslations } from "next-intl";
import { useEffect } from "react";

import { useServices } from "@/entities/service";
import { routes } from "@/shared/config";
import { DataTable } from "@/shared/ui/data-table";
import { EmptyState } from "@/shared/ui/empty-state";
import { ErrorNotice } from "@/shared/ui/error-notice";
import { PageHeader } from "@/shared/ui/page-header";
import { Button, Skeleton } from "@/shared/ui/primitives";
import { SectionCard } from "@/shared/ui/section-card";

import { useServiceColumns } from "./service-columns";

export const LEGACY_EDIT_PARAMETER = "edit";

export function ServicesScreen() {
  const t = useTranslations();
  const router = useRouter();
  const services = useServices();
  const legacyEdit = useSearchParams().get(LEGACY_EDIT_PARAMETER);
  useEffect(() => {
    if (legacyEdit) {
      router.replace(routes.editService(legacyEdit));
    }
  }, [legacyEdit, router]);
  const revision = services.data?.revision ?? null;
  const columns = useServiceColumns(revision);
  const add = (
    <Button asChild>
      <Link href={routes.newService}>
        <Plus aria-hidden />
        {t("services.add")}
      </Link>
    </Button>
  );
  return (
    <div className="grid gap-8">
      <PageHeader title={t("services.title")} description={t("services.subtitle")} actions={add} />
      {services.error && !services.data ? (
        <ErrorNotice title={t("errors.loadFailed")} description={services.error.message} onRetry={() => void services.refetch()} />
      ) : null}
      {services.data ? (
        <SectionCard flush>
          <DataTable
            columns={columns}
            rows={services.data.data.services}
            rowKey={(service) => service.id}
            empty={<EmptyState icon={Server} title={t("services.empty")} description={t("services.emptyHint")} action={add} />}
          />
        </SectionCard>
      ) : services.error ? null : (
        <Skeleton className="h-64 w-full" aria-busy="true" />
      )}
    </div>
  );
}
