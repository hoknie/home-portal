"use client";

import { Server } from "lucide-react";
import { useTranslations } from "next-intl";

import { ServiceCard, type ServiceView } from "@/entities/service";
import { EmptyState } from "@/shared/ui/empty-state";

import { groupServices } from "../../model/grouping";
import type { ServicesSettings } from "../../model/registry";

export function ServicesWidget({
  settings,
  services,
  scope = "private",
}: {
  settings: ServicesSettings;
  services: ServiceView[];
  scope?: "private" | "public";
}) {
  const t = useTranslations("widgets.services");
  if (services.length === 0) {
    return <EmptyState icon={Server} title={t("empty")} description={t("emptyHint")} />;
  }
  const groups = groupServices(services, settings.groups);
  const showHeadings = groups.length > 1 || groups[0]?.name !== null;
  return (
    <div className="grid gap-6">
      {groups.map((group) => (
        <section key={group.name ?? ""} className="grid gap-3">
          {showHeadings ? (
            <h3 className="text-sm font-medium text-muted-foreground">{group.name ?? t("ungrouped")}</h3>
          ) : null}
          <div className="grid gap-3 @lg:grid-cols-2 @4xl:grid-cols-3">
            {group.services.map((service) => (
              <ServiceCard key={service.id} service={service} scope={scope} />
            ))}
          </div>
        </section>
      ))}
    </div>
  );
}
