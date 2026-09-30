"use client";

import { useSearchParams } from "next/navigation";
import { useTranslations } from "next-intl";
import { useState } from "react";

import { CATEGORY_OF, CATEGORY_PARAMETER, MODULE_CATEGORIES, type ModuleCategory, categoryParameter, useModules } from "@/entities/module";
import { routes } from "@/shared/config";
import { replaceAddress } from "@/shared/lib/navigation";
import { ErrorNotice } from "@/shared/ui/error-notice";
import { useTrail } from "@/shared/lib/breadcrumbs";
import { PageHeader } from "@/shared/ui/page-header";
import { Button, Skeleton } from "@/shared/ui/primitives";

import { ModuleCard } from "./module-card";

export function addressOf(category: ModuleCategory | null) {
  return category === null ? routes.adminModules : `${routes.adminModules}?${CATEGORY_PARAMETER}=${category}`;
}

export function ModulesScreen() {
  const trail = useTrail();
  const t = useTranslations("modules");
  const modules = useModules();
  const data = modules.data?.data;
  const asked = categoryParameter(useSearchParams().get(CATEGORY_PARAMETER));
  const [category, setCategory] = useState(asked);
  const choose = (next: ModuleCategory | null) => {
    setCategory(next);
    replaceAddress(addressOf(next));
  };
  const shown = data?.modules.filter((module) => category === null || CATEGORY_OF[module.name] === category) ?? [];
  return (
    <div className="grid gap-8">
      <PageHeader breadcrumbs={trail.of(trail.section("modules"))} title={t("title")} description={t("subtitle")} />
      {modules.error && !data ? (
        <ErrorNotice title={t("loadFailed")} description={modules.error.message} onRetry={() => void modules.refetch()} />
      ) : null}
      <div role="group" aria-label={t("categoryFilter")} className="flex flex-wrap gap-2">
        {[null, ...MODULE_CATEGORIES].map((option) => (
          <Button key={option ?? "all"} type="button" size="sm" variant={option === category ? "default" : "outline"} aria-pressed={option === category} onClick={() => choose(option)}>
            {t(`categories.${option ?? "all"}`)}
          </Button>
        ))}
      </div>
      {data ? (
        <div className="grid gap-6 lg:grid-cols-2">
          {shown.map((module) => (
            <ModuleCard key={module.name} module={module} modules={data} revision={modules.data?.revision ?? null} />
          ))}
        </div>
      ) : modules.error ? null : (
        <Skeleton className="h-96 w-full" aria-busy="true" />
      )}
    </div>
  );
}
