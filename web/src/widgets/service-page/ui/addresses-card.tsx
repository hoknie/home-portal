"use client";

import { useTranslations } from "next-intl";

import type { Service } from "@/entities/service";
import { Badge } from "@/shared/ui/primitives";
import { SectionCard } from "@/shared/ui/section-card";

export type AddressesCardProps = { service: Service; environment: string | null };

export function AddressesCard({ service, environment }: AddressesCardProps) {
  const t = useTranslations("servicePage.addresses");
  const rows = [
    ...Object.entries(service.addresses).map(([name, address]) => ({ name, address })),
    { name: null, address: service.url },
  ];
  const named = rows.findIndex((row) => row.name !== null && row.name === environment);
  const yours = named >= 0 ? named : rows.length - 1;
  const probed = rows.findIndex((row) => row.address === service.probe_address);
  return (
    <SectionCard title={t("title")} description={t("description")}>
      <ul className="grid gap-2">
        {rows.map((row, index) => (
          <li key={row.name ?? "url"} className="flex flex-wrap items-center gap-2 text-sm" data-environment={row.name ?? "url"}>
            <span className="w-28 shrink-0 text-muted-foreground">{row.name ?? t("default")}</span>
            <a href={row.address} target="_blank" rel="noreferrer" className="min-w-0 font-mono text-xs break-all hover:underline">
              {row.address}
            </a>
            {index === yours ? <Badge variant="secondary">{t("yours")}</Badge> : null}
            {index === probed ? <Badge variant="outline">{t("probed")}</Badge> : null}
          </li>
        ))}
      </ul>
    </SectionCard>
  );
}
