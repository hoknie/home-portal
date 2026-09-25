"use client";

import { useTranslations } from "next-intl";
import { Controller, type UseFormReturn } from "react-hook-form";

import type { Catalogue } from "@/entities/automation";
import { FormField } from "@/shared/ui/form-field";
import { Input, Label, Switch } from "@/shared/ui/primitives";
import { SectionCard } from "@/shared/ui/section-card";

import { eventOf } from "../model/events";
import type { AutomationForm } from "../model/form";
import { ChoiceList } from "./choice-list";

type ListName = "services" | "from" | "to" | "users" | "environments" | "webhooks";

export type FiltersCardProps = { form: UseFormReturn<AutomationForm>; catalogue: Catalogue };

export function FiltersCard({ form, catalogue }: FiltersCardProps) {
  const t = useTranslations();
  const filters = eventOf(catalogue, form.watch("event")).filters;
  const errors = form.formState.errors;
  const states = catalogue.states.map((state) => ({ value: state, label: t(`status.${state}` as Parameters<typeof t>[0]) }));
  const options: Record<ListName, { value: string; label: string }[]> = {
    services: catalogue.choices.services.map((service) => ({ value: service.id, label: service.name })),
    from: states,
    to: states,
    users: catalogue.choices.users.map((user) => ({ value: user, label: user })),
    environments: catalogue.choices.environments.map((environment) => ({ value: environment, label: environment })),
    webhooks: catalogue.choices.webhooks.map((webhook) => ({ value: webhook.id, label: webhook.name })),
  };
  const lists = (["services", "from", "to", "users", "environments", "webhooks"] as const).filter((name) => filters.includes(name));
  return (
    <SectionCard title={t("automationBuilder.onlyIf")} description={t("automationBuilder.onlyIfDescription")}>
      <div className="grid gap-5">
        {lists.length === 0 ? <p className="text-sm text-muted-foreground">{t("automationBuilder.noFilters")}</p> : null}
        <div className="grid gap-5 sm:grid-cols-2">
          {lists.map((name) => (
            <Controller
              key={name}
              control={form.control}
              name={name}
              render={({ field }) => (
                <ChoiceList
                  id={`automation-${name}`}
                  legend={t(`automationBuilder.filters.${name}`)}
                  hint={t("automationBuilder.anyHint")}
                  choices={options[name]}
                  values={field.value}
                  onChange={field.onChange}
                  error={errors[name]?.message}
                />
              )}
            />
          ))}
        </div>
        {filters.includes("from_unknown") ? (
          <div className="flex items-start justify-between gap-4">
            <div className="grid gap-1">
              <Label htmlFor="automation-from-unknown">{t("automationBuilder.filters.from_unknown")}</Label>
              <p className="text-xs text-muted-foreground">{t("automationBuilder.fromUnknownHint")}</p>
            </div>
            <Controller
              control={form.control}
              name="from_unknown"
              render={({ field }) => <Switch id="automation-from-unknown" checked={field.value} onCheckedChange={field.onChange} />}
            />
          </div>
        ) : null}
        <FormField
          id="automation-cooldown"
          label={t("automationBuilder.cooldown")}
          hint={t("automationBuilder.cooldownHint")}
          error={errors.cooldown_seconds?.message}
        >
          <Input id="automation-cooldown" type="number" min={0} inputMode="numeric" {...form.register("cooldown_seconds", { valueAsNumber: true })} />
        </FormField>
      </div>
    </SectionCard>
  );
}
