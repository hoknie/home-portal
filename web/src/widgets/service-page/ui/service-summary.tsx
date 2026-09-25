"use client";

import { ExternalLink, Pencil } from "lucide-react";
import Link from "next/link";
import { useTranslations } from "next-intl";

import { type Service, ServiceIcon, iconOf } from "@/entities/service";
import { ProbeNowButton } from "@/features/probe-now";
import { routes } from "@/shared/config";
import { Button } from "@/shared/ui/primitives";
import { RelativeTime } from "@/shared/ui/relative-time";
import { StatusBadge } from "@/shared/ui/status-badge";

export function ServiceSummary({ service }: { service: Service }) {
  const t = useTranslations("servicePage");
  return (
    <header className="glass-panel grid gap-4 rounded-xl p-5 sm:grid-cols-[1fr_auto] sm:items-start">
      <div className="flex min-w-0 items-start gap-4">
        <span className="flex size-14 shrink-0 items-center justify-center rounded-xl border border-glass-edge bg-glass-tint">
          <ServiceIcon {...iconOf(service)} />
        </span>
        <div className="grid min-w-0 gap-1.5">
          {service.group ? <p className="text-xs tracking-wide text-muted-foreground uppercase">{service.group}</p> : null}
          <h1 className="truncate text-2xl font-semibold tracking-tight">{service.name}</h1>
          {service.description ? <p className="text-sm text-muted-foreground">{service.description}</p> : null}
          <div className="flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
            <StatusBadge state={service.status.state} />
            <span>{t("since")}</span>
            <RelativeTime moment={service.status.since} />
          </div>
        </div>
      </div>
      <div className="flex flex-wrap items-start gap-2 sm:justify-end">
        <Button asChild size="sm">
          <a href={service.address || service.url} target="_blank" rel="noreferrer">
            <ExternalLink className="size-4" aria-hidden />
            {t("open")}
          </a>
        </Button>
        <ProbeNowButton service={service} />
        <Button asChild variant="ghost" size="sm">
          <Link href={routes.editService(service.id)}>
            <Pencil className="size-4" aria-hidden />
            {t("edit")}
          </Link>
        </Button>
      </div>
    </header>
  );
}
