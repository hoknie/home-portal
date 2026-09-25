"use client";

import { useTranslations } from "next-intl";

import { type Service, diagnosisMessage } from "@/entities/service";
import { KvList, KvRow } from "@/shared/ui/kv-list";
import { RelativeTime } from "@/shared/ui/relative-time";
import { SectionCard } from "@/shared/ui/section-card";

export function ProbeCard({ service }: { service: Service }) {
  const t = useTranslations();
  const probe = service.probe;
  const status = service.status;
  const message = status.state === "up" || status.state === "degraded" ? null : diagnosisMessage(status.diagnosis);
  const target =
    probe.kind === "http"
      ? `${service.probe_address.replace(/\/$/, "")}${probe.path}`
      : probe.kind === "tcp" && probe.port !== null
        ? `${service.probe_address} : ${probe.port}`
        : service.probe_address;
  return (
    <SectionCard title={t("servicePage.probe.title")}>
      <div className="grid gap-4">
        {message ? (
          <div role="alert" className="grid gap-1 rounded-lg border border-status-down/30 bg-status-down/10 p-3 text-sm" data-diagnosis={status.diagnosis}>
            <p className="font-medium">{t(message.title)}</p>
            <p className="text-muted-foreground">{t(message.action)}</p>
            {status.last_error ? <p className="font-mono text-xs break-all text-muted-foreground">{status.last_error}</p> : null}
          </div>
        ) : null}
        <KvList>
          <KvRow label={t("servicePage.probe.kind")}>
            {probe.enabled ? t(`servicePage.probe.kinds.${probe.kind}`) : t("servicePage.probe.disabled")}
          </KvRow>
          <KvRow label={t("servicePage.probe.target")}>
            <span className="font-mono text-xs break-all">{target}</span>
          </KvRow>
          <KvRow label={t("servicePage.probe.every")}>{t("servicePage.probe.everyValue", { seconds: probe.every_seconds })}</KvRow>
          <KvRow label={t("servicePage.probe.timeout")}>{t("servicePage.probe.timeoutValue", { seconds: probe.timeout_seconds })}</KvRow>
          <KvRow label={t("servicePage.probe.degraded")}>{t("common.milliseconds", { value: probe.degraded_after_milliseconds })}</KvRow>
          <KvRow label={t("servicePage.probe.checked")}>
            <RelativeTime moment={status.checked_at} fallback={t("servicePage.probe.never")} />
          </KvRow>
          <KvRow label={t("servicePage.probe.latency")}>
            {status.latency_milliseconds !== null ? t("common.milliseconds", { value: status.latency_milliseconds }) : t("servicePage.history.noData")}
          </KvRow>
          <KvRow label={t("servicePage.probe.lastOk")}>
            <RelativeTime moment={status.last_ok_at} fallback={t("servicePage.probe.never")} />
          </KvRow>
        </KvList>
      </div>
    </SectionCard>
  );
}
