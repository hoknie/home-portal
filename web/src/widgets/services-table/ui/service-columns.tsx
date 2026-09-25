"use client";

import { Pencil } from "lucide-react";
import { useTranslations } from "next-intl";
import Link from "next/link";

import { DeleteServiceButton } from "@/features/delete-service";
import { type Service, ServiceIcon, diagnosisMessage, iconOf } from "@/entities/service";
import { routes } from "@/shared/config";
import type { Column } from "@/shared/ui/data-table";
import { Button } from "@/shared/ui/primitives";
import { StatusBadge } from "@/shared/ui/status-badge";

export function useServiceColumns(revision: string | null): Column<Service>[] {
  const t = useTranslations();
  return [
    {
      key: "name",
      header: t("services.columns.name"),
      cell: (service) => (
        <div className="flex items-center gap-3">
          <span className="flex size-8 items-center justify-center rounded-md border border-glass-edge bg-glass-tint text-accent-foreground">
            <ServiceIcon {...iconOf(service)} size="sm" />
          </span>
          <div className="min-w-0">
            <Link href={routes.service(service.id)} className="block truncate font-medium hover:underline">
              {service.name}
            </Link>
            <p className="truncate font-mono text-xs text-muted-foreground">{service.id}</p>
          </div>
        </div>
      ),
    },
    { key: "group", header: t("services.columns.group"), hideBelow: "md", cell: (service) => service.group ?? "" },
    {
      key: "url",
      header: t("services.columns.url"),
      hideBelow: "lg",
      cell: (service) => (
        <a href={service.url} target="_blank" rel="noreferrer" className="font-mono text-xs text-muted-foreground hover:text-foreground hover:underline">
          {service.url}
        </a>
      ),
    },
    {
      key: "status",
      header: t("services.columns.status"),
      cell: (service) => {
        const failed = service.status.state === "down" || service.status.state === "unreadable";
        const message = failed ? diagnosisMessage(service.status.diagnosis) : null;
        return (
          <div className="grid justify-items-start gap-1">
            <StatusBadge state={service.status.state} />
            {message ? (
              <span className="text-xs text-muted-foreground" title={t(message.action)} data-diagnosis={service.status.diagnosis}>
                {t(message.title)}
              </span>
            ) : null}
          </div>
        );
      },
    },
    {
      key: "latency",
      header: t("services.columns.latency"),
      hideBelow: "sm",
      align: "end",
      cell: (service) =>
        service.status.latency_milliseconds === null ? "" : t("common.milliseconds", { value: service.status.latency_milliseconds }),
    },
    {
      key: "actions",
      header: t("services.columns.actions"),
      align: "end",
      cell: (service) => (
        <div className="flex justify-end gap-1">
          <Button asChild variant="ghost" size="icon">
            <Link href={routes.editService(service.id)} aria-label={t("common.edit")}>
              <Pencil aria-hidden />
            </Link>
          </Button>
          <DeleteServiceButton service={service} revision={revision} />
        </div>
      ),
    },
  ];
}
