"use client";

import { useFormatter, useTranslations } from "next-intl";
import { useEffect, useState } from "react";
import type { UseFormReturn } from "react-hook-form";

import { useSchedule } from "@/entities/automation";
import { ValidationError } from "@/shared/api";
import { FormField } from "@/shared/ui/form-field";
import { Input } from "@/shared/ui/primitives";

import { CRON_PRESETS, type CronPreset, WEEKDAYS, presetCron } from "../model/cron";
import type { AutomationForm } from "../model/form";
import { SELECT } from "./select";

export const CRON_DEBOUNCE_MILLISECONDS = 400;

function useSettled(value: string) {
  const [settled, setSettled] = useState(value);
  useEffect(() => {
    const timer = window.setTimeout(() => setSettled(value), CRON_DEBOUNCE_MILLISECONDS);
    return () => window.clearTimeout(timer);
  }, [value]);
  return settled;
}

export function CronField({ form }: { form: UseFormReturn<AutomationForm> }) {
  const t = useTranslations("automationBuilder");
  const format = useFormatter();
  const cron = form.watch("cron");
  const settled = useSettled(cron.trim());
  const schedule = useSchedule(settled);
  const [preset, setPreset] = useState<CronPreset | "custom">("custom");
  const [time, setTime] = useState("03:00");
  const [weekday, setWeekday] = useState(1);
  const apply = (next: CronPreset | "custom", at = time, day = weekday) => {
    setPreset(next);
    if (next !== "custom") {
      form.setValue("cron", presetCron(next, at, day), { shouldDirty: true, shouldValidate: true });
    }
  };
  const serverError = schedule.error instanceof ValidationError ? schedule.error.fields[0]?.message : undefined;
  return (
    <div className="grid gap-4">
      <div className="grid gap-4 sm:grid-cols-3">
        <FormField id="automation-preset" label={t("preset")}>
          <select id="automation-preset" className={SELECT} value={preset} onChange={(event) => apply(event.target.value as CronPreset | "custom")}>
            <option value="custom">{t("presets.custom")}</option>
            {CRON_PRESETS.map((name) => (
              <option key={name} value={name}>
                {t(`presets.${name}`)}
              </option>
            ))}
          </select>
        </FormField>
        {preset === "daily" || preset === "weekly" || preset === "monthly" ? (
          <FormField id="automation-time" label={t("time")}>
            <Input
              id="automation-time"
              type="time"
              value={time}
              onChange={(event) => {
                setTime(event.target.value);
                apply(preset, event.target.value, weekday);
              }}
            />
          </FormField>
        ) : null}
        {preset === "weekly" ? (
          <FormField id="automation-weekday" label={t("weekday")}>
            <select
              id="automation-weekday"
              className={SELECT}
              value={weekday}
              onChange={(event) => {
                const day = Number(event.target.value);
                setWeekday(day);
                apply(preset, time, day);
              }}
            >
              {WEEKDAYS.map((day) => (
                <option key={day} value={day}>
                  {t(`weekdays.${day}`)}
                </option>
              ))}
            </select>
          </FormField>
        ) : null}
      </div>
      <FormField id="automation-cron" label={t("cron")} hint={t("cronHint")} error={form.formState.errors.cron?.message ?? serverError}>
        <Input
          id="automation-cron"
          spellCheck={false}
          autoComplete="off"
          className="font-mono"
          {...form.register("cron", { onChange: () => setPreset("custom") })}
        />
      </FormField>
      {schedule.data && !serverError && settled !== "" ? (
        <div className="grid gap-1">
          <p className="text-sm font-medium">{t("nextTimes", { zone: schedule.data.timezone })}</p>
          <ul aria-label={t("nextTimesList")} className="grid gap-0.5 font-mono text-xs text-muted-foreground">
            {schedule.data.times.map((moment) => (
              <li key={moment}>{format.dateTime(new Date(moment), { dateStyle: "full", timeStyle: "short", timeZone: schedule.data.timezone })}</li>
            ))}
          </ul>
        </div>
      ) : null}
    </div>
  );
}
