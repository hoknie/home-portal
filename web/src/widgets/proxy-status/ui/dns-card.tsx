"use client";

import { TriangleAlert } from "lucide-react";
import { useTranslations } from "next-intl";

import { type Dns, type DnsTransport, answerIn, useDns } from "@/entities/dns-server";
import { CopyLine } from "@/shared/ui/copy-line";
import { ErrorNotice } from "@/shared/ui/error-notice";
import { KvList, KvRow } from "@/shared/ui/kv-list";
import { SectionCard } from "@/shared/ui/section-card";
import { StatusDot } from "@/shared/ui/status-badge";

import { DnsSettingsForm } from "./dns-settings-form";

export const TRANSPORTS = ["plain", "tls", "https"] as const;

function TransportRow({ name, transport }: { name: (typeof TRANSPORTS)[number]; transport: DnsTransport }) {
  const t = useTranslations("dns");
  return (
    <KvRow label={t(`transports.${name}`)}>
      <span role="status" data-transport={name} className="inline-flex flex-wrap items-center gap-2">
        <StatusDot state={transport.listening ? "up" : "down"} />
        {transport.listening ? t("listening") : t("notListening")}
        {transport.address ? <span className="font-mono text-xs">{transport.address}</span> : null}
        {!transport.listening && transport.reason ? <span className="text-xs text-muted-foreground">{transport.reason}</span> : null}
      </span>
    </KvRow>
  );
}

function Answers({ dns }: { dns: Dns }) {
  const t = useTranslations("dns");
  if (dns.names.length === 0) {
    return <p className="text-sm text-muted-foreground">{t("noNames")}</p>;
  }
  return (
    <div className="overflow-x-auto">
      <table className="w-full text-sm">
        <thead>
          <tr className="text-left text-muted-foreground">
            <th className="py-1 pr-4 font-medium">{t("name")}</th>
            {dns.environments.map((environment) => (
              <th key={environment} className="py-1 pr-4 font-medium">
                {environment}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {dns.names.map((entry) => (
            <tr key={entry.name} className="border-t border-glass-edge">
              <td className="py-1 pr-4 font-mono text-xs break-all">{entry.name}</td>
              {dns.environments.map((environment) => {
                const records = answerIn(dns, entry.name, environment);
                return (
                  <td key={environment} className="py-1 pr-4 font-mono text-xs">
                    {records.length === 0 ? <span className="text-muted-foreground">{t("noAnswer")}</span> : records.map((record) => <div key={`${record.type}-${record.value}`}>{record.value}</div>)}
                  </td>
                );
              })}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

export function DnsCard() {
  const t = useTranslations("dns");
  const common = useTranslations();
  const dns = useDns();
  const data = dns.data?.data;
  if (!data) {
    return dns.error ? <ErrorNotice title={common("errors.loadFailed")} description={dns.error.message} onRetry={() => void dns.refetch()} /> : null;
  }
  return (
    <SectionCard title={t("title")} description={t("description")}>
      <div className="grid gap-6">
        <KvList>
          {TRANSPORTS.map((name) => (
            <TransportRow key={name} name={name} transport={data[name]} />
          ))}
          <KvRow label={t("zones")}>{data.zones.length === 0 ? t("noZones") : data.zones.map((zone) => zone.apex).join(", ")}</KvRow>
        </KvList>
        {data.last_error ? <ErrorNotice title={t("lastError")} description={data.last_error} /> : null}
        {data.unaddressed.map((environment) => (
          <p key={environment} role="alert" className="flex items-start gap-2 text-sm">
            <TriangleAlert className="mt-0.5 size-4 shrink-0 text-status-degraded" aria-hidden />
            {t("unaddressed", { environment })}
          </p>
        ))}
        <Answers dns={data} />
        {data.tls_host ? <CopyLine label={t("tlsHost")} text={data.tls_host} /> : null}
        {data.doh_url ? <CopyLine label={t("dohUrl")} text={data.doh_url} /> : null}
        <details className="text-sm">
          <summary className="cursor-pointer font-medium">{t("forwardingTitle")}</summary>
          <p className="mt-2 text-muted-foreground">{t("forwarding")}</p>
        </details>
        <DnsSettingsForm dns={data} revision={dns.data?.revision ?? null} />
      </div>
    </SectionCard>
  );
}
