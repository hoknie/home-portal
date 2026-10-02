"use client";

import { Bell, Globe, Route, Settings2, Users, Waypoints, Webhook, Workflow, type LucideIcon } from "lucide-react";
import Link from "next/link";
import { useTranslations } from "next-intl";

import { CATEGORY_OF, type Module, type ModuleName, type Modules } from "@/entities/module";
import { MODULE_PAGES } from "@/shared/config";
import { Badge, Button } from "@/shared/ui/primitives";
import { SectionCard } from "@/shared/ui/section-card";

import { ModuleSwitch } from "./module-switch";

const ICONS: Record<ModuleName, LucideIcon> = {
  proxy: Waypoints,
  dns: Globe,
  automations: Workflow,
  webhooks: Webhook,
  users: Users,
  workflows: Route,
  notifications: Bell,
};

export type ModuleCardProps = { module: Module; modules: Modules; revision: string | null };

export function ModuleCard({ module, modules, revision }: ModuleCardProps) {
  const t = useTranslations("modules");
  const Icon = ICONS[module.name];
  const listed = (names: ModuleName[]) => names.map((name) => t(`names.${name}`)).join(", ");
  return (
    <SectionCard
      hoverable
      title={t(`names.${module.name}`)}
      description={t(`descriptions.${module.name}`)}
      badge={
        <Badge variant={module.enabled ? "default" : "outline"}>
          <Icon aria-hidden />
          {t(module.enabled ? "on" : "off")}
        </Badge>
      }
    >
      <div className="grid gap-4">
        <p className="text-xs font-medium text-muted-foreground" data-category={CATEGORY_OF[module.name]}>
          {t(`categories.${CATEGORY_OF[module.name]}`)}
        </p>
        {module.requires.length > 0 || module.required_by.length > 0 ? (
          <ul className="grid gap-1 text-sm text-muted-foreground">
            {module.requires.length > 0 ? <li>{t("requires", { names: listed(module.requires) })}</li> : null}
            {module.required_by.length > 0 ? <li>{t("requiredBy", { names: listed(module.required_by) })}</li> : null}
          </ul>
        ) : null}
        <div className="flex flex-wrap items-start justify-between gap-3">
          <ModuleSwitch module={module} modules={modules} revision={revision} />
          <Button asChild variant="outline" size="sm">
            <Link href={MODULE_PAGES[module.name]}>
              <Settings2 aria-hidden />
              {t("configure")}
            </Link>
          </Button>
        </div>
      </div>
    </SectionCard>
  );
}
