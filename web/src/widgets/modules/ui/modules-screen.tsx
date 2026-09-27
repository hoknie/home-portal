"use client";

import { useTranslations } from "next-intl";

import { useModules } from "@/entities/module";
import { ErrorNotice } from "@/shared/ui/error-notice";
import { PageHeader } from "@/shared/ui/page-header";
import { Skeleton } from "@/shared/ui/primitives";

import { ModuleCard } from "./module-card";

export function ModulesScreen() {
  const t = useTranslations("modules");
  const modules = useModules();
  const data = modules.data?.data;
  return (
    <div className="grid gap-8">
      <PageHeader title={t("title")} description={t("subtitle")} />
      {modules.error && !data ? (
        <ErrorNotice title={t("loadFailed")} description={modules.error.message} onRetry={() => void modules.refetch()} />
      ) : null}
      {data ? (
        <div className="grid gap-6 lg:grid-cols-2">
          {data.modules.map((module) => (
            <ModuleCard key={module.name} module={module} modules={data} revision={modules.data?.revision ?? null} />
          ))}
        </div>
      ) : modules.error ? null : (
        <Skeleton className="h-96 w-full" aria-busy="true" />
      )}
    </div>
  );
}
