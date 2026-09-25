"use client";

import { ExternalLink } from "lucide-react";
import Link from "next/link";
import { useFormatter, useNow, useTranslations } from "next-intl";

import { diagnosisMessage } from "../model/diagnosis";
import { iconOf } from "../model/icons";
import type { ServiceView } from "../model/schema";
import { ServiceIcon } from "./service-icon";
import { STATUS_REFRESH_MILLISECONDS, routes } from "@/shared/config";
import { StatusBadge } from "@/shared/ui/status-badge";

export type ServiceCardProps = { service: ServiceView; scope?: "private" | "public" };

const CARD =
  "glass-panel group relative flex flex-col gap-3 rounded-xl p-4 transition hover:-translate-y-0.5 hover:border-primary/40 focus-within:ring-2 focus-within:ring-ring";

export function ServiceCard({ service, scope = "private" }: ServiceCardProps) {
  const t = useTranslations();
  const address = service.address || service.url || "";
  const body = (
    <>
      <div className="flex items-start gap-3">
        <span className="flex size-10 shrink-0 items-center justify-center rounded-lg border border-glass-edge bg-glass-tint text-accent-foreground">
          <ServiceIcon {...iconOf(service, scope)} />
        </span>
        <div className="min-w-0 flex-1">
          <p className="truncate font-medium">{service.name}</p>
          <p className="truncate text-xs text-muted-foreground">{service.description ?? address}</p>
        </div>
        {scope === "public" ? (
          <ExternalLink className="size-4 shrink-0 text-muted-foreground opacity-0 transition group-hover:opacity-100" aria-hidden />
        ) : null}
      </div>
      <CardStatus service={service} />
    </>
  );
  if (scope === "public") {
    return (
      <a href={address} target="_blank" rel="noreferrer" className={`${CARD} focus-visible:outline-none`}>
        {body}
      </a>
    );
  }
  return (
    <div className={CARD}>
      <Link
        href={routes.service(service.id)}
        className="flex flex-col gap-3 outline-none after:absolute after:inset-0 after:rounded-xl"
      >
        {body}
      </Link>
      <a
        href={address}
        target="_blank"
        rel="noreferrer"
        aria-label={t("services.openService", { name: service.name })}
        className="absolute top-3 right-3 z-10 flex size-8 items-center justify-center rounded-md text-muted-foreground transition hover:bg-glass-tint hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
      >
        <ExternalLink className="size-4" aria-hidden />
      </a>
    </div>
  );
}

function CardStatus({ service }: { service: ServiceView }) {
  const t = useTranslations();
  const format = useFormatter();
  const now = useNow({ updateInterval: STATUS_REFRESH_MILLISECONDS });
  const status = service.status;
  if (!status) {
    return null;
  }
  const message = status.state === "down" || status.state === "unreadable" ? diagnosisMessage(status.diagnosis) : null;
  const explained = [message ? t(message.title) : null, status.last_error].filter(Boolean).join(": ");
  return (
    <div className="flex items-center justify-between gap-2 text-xs text-muted-foreground">
      <StatusBadge state={status.state} />
      <span className="tabular-nums" title={explained || undefined}>
        {status.latency_milliseconds !== null
          ? t("common.milliseconds", { value: status.latency_milliseconds })
          : status.checked_at
            ? format.relativeTime(new Date(status.checked_at), now)
            : t("status.notChecked")}
      </span>
    </div>
  );
}
