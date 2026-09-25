"use client";

import { useTranslations } from "next-intl";

import { type Automation, type Catalogue, messageKeyOf } from "@/entities/automation";

export function TriggerText({ automation, catalogue }: { automation: Automation; catalogue: Catalogue | undefined }) {
  const t = useTranslations();
  const { when } = automation;
  if (when.event === "schedule") {
    return <span className="font-mono text-xs">{t("automations.triggers.schedule", { cron: when.cron ?? "" })}</span>;
  }
  const event = t(`automationEvents.${messageKeyOf(when.event)}.title` as Parameters<typeof t>[0]);
  if (when.event !== "service.status-changed") {
    return <span>{event}</span>;
  }
  const names = when.services.map((id) => catalogue?.choices.services.find((service) => service.id === id)?.name ?? id);
  const services = names.length === 0 ? t("automations.triggers.anyService") : names.join(", ");
  const states = when.to.map((state) => t(`status.${state}` as Parameters<typeof t>[0]).toLowerCase());
  return (
    <span>
      {states.length === 0
        ? t("automations.triggers.statusChanged", { services })
        : t("automations.triggers.statusBecomes", { services, states: states.join(", ") })}
    </span>
  );
}
