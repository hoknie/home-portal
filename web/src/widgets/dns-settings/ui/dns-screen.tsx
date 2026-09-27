"use client";

import { useTranslations } from "next-intl";

import { useDns } from "@/entities/dns-server";
import { ModuleOffNotice } from "@/shared/ui/module-off-notice";
import { PageHeader } from "@/shared/ui/page-header";

import { DnsCard } from "./dns-card";

export function DnsScreen() {
  const t = useTranslations();
  const dns = useDns();
  const data = dns.data?.data;
  return (
    <div className="grid gap-8">
      <PageHeader title={t("dns.title")} description={t("dns.description")} />
      {data && !data.enabled ? <ModuleOffNotice name={t("modules.names.dns")} /> : null}
      <DnsCard />
    </div>
  );
}
