"use client";

import { useTranslations } from "next-intl";
import { Controller, type UseFormReturn } from "react-hook-form";

import { PROBE_KINDS, type ServiceForm } from "@/entities/service";
import { FormField } from "@/shared/ui/form-field";
import { Input, Label, Switch } from "@/shared/ui/primitives";

const SELECT =
  "h-9 w-full rounded-md border border-input bg-glass-tint px-3 text-sm shadow-xs outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50";

export function ProbeFields({ form }: { form: UseFormReturn<ServiceForm> }) {
  const t = useTranslations();
  const errors = form.formState.errors.probe;
  const number = { valueAsNumber: true } as const;
  const kind = form.watch("probe.kind");
  return (
    <div className="grid gap-4">
      <div className="flex items-center justify-between gap-4">
        <Label htmlFor="probe-enabled">{t("serviceForm.probeEnabled")}</Label>
        <Controller
          control={form.control}
          name="probe.enabled"
          render={({ field }) => <Switch id="probe-enabled" checked={field.value} onCheckedChange={field.onChange} />}
        />
      </div>
      <div className="grid gap-4 sm:grid-cols-2">
        <FormField id="probe-kind" label={t("serviceForm.probeKind")} hint={t(`serviceForm.probeKindHints.${kind}`)}>
          <select id="probe-kind" className={SELECT} {...form.register("probe.kind")}>
            {PROBE_KINDS.map((value) => (
              <option key={value} value={value}>
                {t(`servicePage.probe.kinds.${value}`)}
              </option>
            ))}
          </select>
        </FormField>
        {kind === "http" ? (
          <FormField id="probe-path" label={t("serviceForm.probePath")} error={errors?.path?.message}>
            <Input id="probe-path" {...form.register("probe.path")} />
          </FormField>
        ) : null}
        {kind === "tcp" ? (
          <FormField id="probe-port" label={t("serviceForm.probePort")} hint={t("serviceForm.probePortHint")} optional error={errors?.port?.message}>
            <Input
              id="probe-port"
              type="number"
              inputMode="numeric"
              {...form.register("probe.port", { setValueAs: (value) => (value === "" || value === null ? null : Number(value)) })}
            />
          </FormField>
        ) : null}
      </div>
      <div className="grid gap-4 sm:grid-cols-3">
        <FormField id="probe-every" label={t("serviceForm.probeEvery")} error={errors?.every_seconds?.message}>
          <Input id="probe-every" type="number" inputMode="numeric" {...form.register("probe.every_seconds", number)} />
        </FormField>
        <FormField id="probe-timeout" label={t("serviceForm.probeTimeout")} error={errors?.timeout_seconds?.message}>
          <Input id="probe-timeout" type="number" inputMode="numeric" {...form.register("probe.timeout_seconds", number)} />
        </FormField>
        <FormField id="probe-degraded" label={t("serviceForm.probeDegraded")} error={errors?.degraded_after_milliseconds?.message}>
          <Input id="probe-degraded" type="number" inputMode="numeric" {...form.register("probe.degraded_after_milliseconds", number)} />
        </FormField>
      </div>
    </div>
  );
}
