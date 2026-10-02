"use client";

import { Blocks, Download, ExternalLink } from "lucide-react";
import Link from "next/link";
import { useTranslations } from "next-intl";

import { CaddyControl } from "@/features/caddy-control";
import { ApplyProxyButton } from "@/features/proxy-apply";
import { ProxySettingsForm } from "@/features/proxy-settings-form";
import { type Proxy, type ProxyRoute, useProxy, usesInternal } from "@/entities/proxy";
import { api, routes } from "@/shared/config";
import { DataTable } from "@/shared/ui/data-table";
import { Appear, Button, ErrorNotice, KvList, KvRow, SkeletonForm } from "@/shared/ui/kit";
import { ModuleOffNotice } from "@/shared/ui/module-off-notice";
import { useTrail } from "@/shared/lib/breadcrumbs";
import { PageHeader } from "@/shared/ui/page-header";
import { RelativeTime } from "@/shared/ui/relative-time";
import { SectionCard } from "@/shared/ui/section-card";
import { StatusDot } from "@/shared/ui/status-badge";


function stateOf(proxy: Proxy) {
  if (!proxy.reachable) {
    return { dot: "down", label: "unreachable" } as const;
  }
  return proxy.in_sync ? ({ dot: "up", label: "inSync" } as const) : ({ dot: "degraded", label: "outOfSync" } as const);
}

function ProxyState({ proxy }: { proxy: Proxy }) {
  const t = useTranslations("proxy");
  const state = stateOf(proxy);
  return (
    <SectionCard title={t("state")} actions={<ApplyProxyButton />}>
      <KvList>
        <KvRow label={t("caddy")}>
          <span role="status" className="inline-flex items-center gap-2">
            <StatusDot state={state.dot} />
            {t(state.label)}
          </span>
        </KvRow>
        <KvRow label={t("admin")}>
          <span className="font-mono">{proxy.admin}</span>
        </KvRow>
        <KvRow label={t("lastApplied")}>
          <RelativeTime moment={proxy.last_applied_at} fallback={t("never")} />
        </KvRow>
        {proxy.last_error ? (
          <KvRow label={t("lastError")}>
            <span className="font-mono text-xs text-status-down">{proxy.last_error}</span>
          </KvRow>
        ) : null}
      </KvList>
    </SectionCard>
  );
}

function HostsTable({ routes: rows }: { routes: ProxyRoute[] }) {
  const t = useTranslations("proxy");
  const listed = (names: string[]) => (names.length === 0 ? t("none") : names.join(", "));
  return (
    <SectionCard title={t("hosts")} flush>
      <DataTable
        rows={rows}
        rowKey={(route) => route.host}
        columns={[
          {
            key: "host",
            header: t("host"),
            cell: (route) => (
              <a href={route.address} target="_blank" rel="noreferrer" className="inline-flex items-center gap-1 font-medium hover:underline">
                {route.address.replace("https://", "")}
                <ExternalLink className="size-3" aria-hidden />
              </a>
            ),
          },
          {
            key: "service",
            header: t("service"),
            cell: (route) =>
              route.service ? (
                <a href={routes.service(route.service)} className="hover:underline">
                  {route.service}
                </a>
              ) : (
                t("portal")
              ),
          },
          { key: "upstream", header: t("upstream"), hideBelow: "md", cell: (route) => <span className="font-mono text-xs break-all whitespace-normal">{route.upstream}</span> },
          { key: "tls", header: t("tls"), cell: (route) => t(`tlsModes.${route.tls}`) },
          { key: "auth", header: t("auth"), hideBelow: "sm", cell: (route) => listed(route.auth) },
          { key: "environments", header: t("environments"), hideBelow: "lg", cell: (route) => listed(route.environments) },
        ]}
      />
    </SectionCard>
  );
}

function RootCertificate() {
  const t = useTranslations("proxy");
  return (
    <SectionCard title={t("rootCertificate")} description={t("rootCertificateHint")}>
      <a href={api.proxyRootCertificate} download className="inline-flex items-center gap-2 text-sm font-medium hover:underline">
        <Download className="size-4" aria-hidden />
        {t("download")}
      </a>
    </SectionCard>
  );
}

export function ProxyScreen() {
  const trail = useTrail();
  const t = useTranslations();
  const proxy = useProxy();
  const data = proxy.data?.data;
  return (
    <div className="grid gap-8">
      <PageHeader breadcrumbs={trail.of(trail.section("proxy"))}
        title={t("proxy.title")}
        description={t("proxy.subtitle")}
        actions={
          <Button asChild variant="outline" size="sm">
            <Link href={routes.adminModules}>
              <Blocks aria-hidden />
              {t("nav.modules")}
            </Link>
          </Button>
        }
      />
      {proxy.error && !data ? (
        <ErrorNotice title={t("errors.loadFailed")} description={proxy.error.message} onRetry={() => void proxy.refetch()} />
      ) : null}
      {data && !data.enabled ? <ModuleOffNotice name={t("modules.names.proxy")} /> : null}
      {data?.enabled ? (
        <div className="grid gap-6">
          <ProxyState proxy={data} />
          <CaddyControl proxy={data} revision={proxy.data?.revision ?? null} />
          <HostsTable routes={data.routes} />
          {usesInternal(data) ? <RootCertificate /> : null}
        </div>
      ) : null}
      {data && !data.enabled ? <CaddyControl proxy={data} revision={proxy.data?.revision ?? null} /> : null}
      {data ? (
        <Appear>
          <SectionCard title={t("proxy.settings.title")} description={t("proxy.settings.description")}>
            <ProxySettingsForm proxy={data} revision={proxy.data?.revision ?? null} />
          </SectionCard>
        </Appear>
      ) : null}
      {!data && !proxy.error ? <SkeletonForm fields={6} /> : null}
    </div>
  );
}
