"use client";

import { SearchX } from "lucide-react";
import Link from "next/link";
import { useRouter, useSearchParams } from "next/navigation";
import { useLocale, useTranslations } from "next-intl";

import { ServiceForm } from "@/features/service-form";
import { useEnvironment } from "@/entities/environment";
import { groupsOf, useServices } from "@/entities/service";
import { routes } from "@/shared/config";
import { Appear, Button, EmptyState, Loaded, SkeletonForm } from "@/shared/ui/kit";
import { useTrail } from "@/shared/lib/breadcrumbs";
import { PageHeader } from "@/shared/ui/page-header";

export const ID_PARAMETER = "id";

export type ServiceEditorScreenProps = { mode: "new" | "edit" };

export function ServiceEditorScreen({ mode }: ServiceEditorScreenProps) {
  const t = useTranslations();
  const trail = useTrail();
  const locale = useLocale();
  const router = useRouter();
  const id = useSearchParams().get(ID_PARAMETER) ?? "";
  const services = useServices();
  const environment = useEnvironment();
  const title = t(mode === "new" ? "serviceForm.addTitle" : "serviceForm.editTitle");
  const serviceName = services.data?.data.services.find((candidate) => candidate.id === id)?.name ?? id;
  const header = <PageHeader breadcrumbs={trail.of(trail.section("services"), { label: mode === "new" ? t("breadcrumbs.new.service") : serviceName })} title={title} description={t("serviceForm.description")} />;
  if (!services.data || !environment.data) {
    return (
      <div className="grid gap-8">
        {header}
        <Loaded.all queries={[services, environment]} skeleton={<SkeletonForm fields={6} />}>
          {() => null}
        </Loaded.all>
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
      <Appear>
        <ServiceForm
          key={service?.id ?? "new"}
          service={service}
          environments={environment.data.environments}
          cancelHref={service ? routes.service(service.id) : routes.adminServices}
          revision={services.data.revision}
          taken={all.map((candidate) => candidate.id).filter((candidate) => candidate !== service?.id)}
          groups={groupsOf(all, locale)}
          onSaved={() => router.push(routes.adminServices)}
          onConflict={() => void services.refetch()}
        />
      </Appear>
    </div>
  );
}
