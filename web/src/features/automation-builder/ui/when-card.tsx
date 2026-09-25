"use client";

import { useTranslations } from "next-intl";
import type { UseFormReturn } from "react-hook-form";

import { type Catalogue, type FilterName, messageKeyOf } from "@/entities/automation";
import { FormField } from "@/shared/ui/form-field";
import { SectionCard } from "@/shared/ui/section-card";

import { EVENT_GROUPS, eventOf, groupOf } from "../model/events";
import { CLEARED, type AutomationForm, foreignFilters } from "../model/form";
import { CronField } from "./cron-field";
import { SELECT } from "./select";

export type WhenCardProps = { form: UseFormReturn<AutomationForm>; catalogue: Catalogue };

export function WhenCard({ form, catalogue }: WhenCardProps) {
  const t = useTranslations();
  const event = form.watch("event");
  const change = (next: AutomationForm["event"]) => {
    form.setValue("event", next, { shouldDirty: true });
    for (const name of foreignFilters(eventOf(catalogue, next).filters)) {
      form.setValue(name as FilterName, CLEARED[name] as never, { shouldDirty: true });
    }
    void form.trigger("args");
  };
  const key = messageKeyOf(event);
  return (
    <SectionCard title={t("automationBuilder.when")} description={t("automationBuilder.whenDescription")}>
      <div className="grid gap-4">
        <FormField id="automation-event" label={t("automationBuilder.event")} hint={t(`automationEvents.${key}.description` as Parameters<typeof t>[0])}>
          <select id="automation-event" className={SELECT} value={event} onChange={(change_) => change(change_.target.value as AutomationForm["event"])}>
            {EVENT_GROUPS.map((group) => (
              <optgroup key={group} label={t(`automationBuilder.groups.${group}`)}>
                {catalogue.events
                  .filter((candidate) => groupOf(candidate.name) === group)
                  .map((candidate) => (
                    <option key={candidate.name} value={candidate.name}>
                      {t(`automationEvents.${messageKeyOf(candidate.name)}.title` as Parameters<typeof t>[0])}
                    </option>
                  ))}
              </optgroup>
            ))}
          </select>
        </FormField>
        {event === "schedule" ? <CronField form={form} /> : null}
      </div>
    </SectionCard>
  );
}
