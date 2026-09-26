"use client";

import { Info, RotateCcw } from "lucide-react";
import { useTranslations } from "next-intl";

import { NetworkForm } from "@/features/network-form";
import { RestartPortalButton } from "@/features/restart-portal";
import { useNetwork } from "@/entities/network";
import { ErrorNotice } from "@/shared/ui/error-notice";
import { KvList, KvRow } from "@/shared/ui/kv-list";
import { PageHeader } from "@/shared/ui/page-header";
import { Skeleton } from "@/shared/ui/primitives";
import { SectionCard } from "@/shared/ui/section-card";

export function NetworkScreen() {
  const t = useTranslations();
  const network = useNetwork();
  const data = network.data?.data;
  return (
    <div className="grid gap-8">
      <PageHeader title={t("network.title")} description={t("network.subtitle")} />
      {network.error && !data ? (
        <ErrorNotice title={t("errors.loadFailed")} description={network.error.message} onRetry={() => void network.refetch()} />
      ) : null}
      {data ? (
        <div className="grid gap-6">
          {data.restart_required ? (
            <div role="status" className="flex flex-wrap items-start gap-3 rounded-xl border border-status-degraded/40 bg-status-degraded/10 p-4 text-sm">
              <RotateCcw className="mt-0.5 size-4 shrink-0" aria-hidden />
              <span className="min-w-0 flex-1">{t("network.restartRequired")}</span>
              <RestartPortalButton network={data} />
            </div>
          ) : null}
          <SectionCard title={t("network.effective")}>
            <KvList>
              <KvRow label={t("network.effectiveAddress")}>
                <span className="font-mono">{data.effective.address}</span>
              </KvRow>
            </KvList>
            {data.effective.overridden ? (
              <p className="mt-4 flex items-start gap-2 text-sm text-muted-foreground">
                <Info className="mt-0.5 size-4 shrink-0" aria-hidden />
                {t("network.overridden")}
              </p>
            ) : null}
          </SectionCard>
          <SectionCard title={t("network.configured")}>
            <NetworkForm configured={data.configured} revision={network.data?.revision ?? null} />
          </SectionCard>
          <SectionCard title={t("network.interfaces")} description={t("network.interfacesHint")}>
            <KvList>
              {data.interfaces.map((networkInterface) => (
                <KvRow key={networkInterface.name} label={networkInterface.name}>
                  <span className="font-mono text-xs">{networkInterface.addresses.join(", ")}</span>
                </KvRow>
              ))}
            </KvList>
          </SectionCard>
        </div>
      ) : network.error ? null : (
        <Skeleton className="h-96 w-full" aria-busy="true" />
      )}
    </div>
  );
}
