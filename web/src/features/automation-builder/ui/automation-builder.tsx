"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import Link from "next/link";
import { useTranslations } from "next-intl";
import { useMemo, useState } from "react";
import { Controller, type Path, type UseFormReturn, useForm, useWatch } from "react-hook-form";
import { toast } from "sonner";

import { type Automation, type Catalogue, type Scripts, useSaveAutomation } from "@/entities/automation";
import { ConflictError, ValidationError } from "@/shared/api";
import { routes } from "@/shared/config";
import { useLeaveGuard } from "@/shared/lib/leave-guard";
import { slugOf, uniqueId } from "@/shared/lib/slug";
import { ErrorNotice } from "@/shared/ui/error-notice";
import { FormField } from "@/shared/ui/form-field";
import { Button, Input, Label, Switch } from "@/shared/ui/primitives";
import { SectionCard } from "@/shared/ui/section-card";

import { fieldsOf, withVariables } from "../model/events";
import { type AutomationForm, RESERVED_IDS, automationFormSchema, emptyAutomationForm, formOf, requestOf } from "../model/form";
import type { RunFields } from "../model/run-fields";
import { byField } from "../model/server-errors";
import { CommandPreview } from "./command-preview";
import { FiltersCard } from "./filters-card";
import { RunCard } from "./run-card";
import { TagsField } from "./tags-field";
import { WhenCard } from "./when-card";

export type AutomationBuilderProps = {
  automation: Automation | null;
  revision: string | null;
  taken: string[];
  catalogue: Catalogue;
  scripts: Scripts;
  onSaved: () => void;
  onConflict: () => void;
};

export function AutomationBuilder({ automation, revision, taken, catalogue, scripts, onSaved, onConflict }: AutomationBuilderProps) {
  const t = useTranslations();
  const save = useSaveAutomation();
  const [conflict, setConflict] = useState(false);
  const [saved, setSaved] = useState(false);
  const [idFollowsTitle, setIdFollowsTitle] = useState(automation === null);
  const schema = useMemo(() => automationFormSchema(fieldsOf(catalogue)), [catalogue]);
  const form = useForm<AutomationForm>({
    resolver: zodResolver(schema),
    mode: "onChange",
    defaultValues: automation ? formOf(automation) : emptyAutomationForm,
  });
  useLeaveGuard(form.formState.isDirty && !saved, t("automationBuilder.leave"));
  const values = useWatch({ control: form.control }) as AutomationForm;
  const event = withVariables(catalogue, values.event, values.webhooks);

  const submit = form.handleSubmit(async (current) => {
    setConflict(false);
    try {
      await save.mutateAsync({ id: automation?.id ?? null, body: requestOf(current, event.filters), revision });
      setSaved(true);
      toast.success(t(automation ? "automations.saved" : "automations.created"));
      onSaved();
    } catch (error) {
      if (error instanceof ValidationError) {
        for (const { path, message } of byField(error.fields)) {
          form.setError(path as Path<AutomationForm>, { message });
        }
      } else if (error instanceof ConflictError) {
        setConflict(true);
        onConflict();
      } else {
        toast.error(t("errors.generic"));
      }
    }
  });

  const errors = form.formState.errors;
  const chosen = {
    "service.id": values.services[0],
    "status.from": values.from[0],
    "status.to": values.to[0],
    "user.name": values.users[0],
    "client.environment": values.environments[0],
    "schedule.cron": values.cron || undefined,
    "automation.id": values.id || undefined,
  };
  return (
    <form onSubmit={submit} className="grid gap-6" noValidate>
      {conflict ? <ErrorNotice title={t("errors.conflict")} /> : null}
      <SectionCard title={t("automationBuilder.identity")}>
        <div className="grid gap-4 sm:grid-cols-[1fr_16rem_auto] sm:items-start">
          <FormField id="automation-title" label={t("automationBuilder.title")} error={errors.title?.message}>
            <Input
              id="automation-title"
              autoFocus
              {...form.register("title", {
                onChange: (change: { target: { value: string } }) => {
                  if (idFollowsTitle) {
                    form.setValue("id", uniqueId(slugOf(change.target.value), [...taken, ...RESERVED_IDS]), { shouldDirty: true, shouldValidate: true });
                  }
                },
              })}
            />
          </FormField>
          <FormField id="automation-id" label={t("automationBuilder.id")} hint={t("automationBuilder.idHint")} error={errors.id?.message}>
            <Input
              id="automation-id"
              autoComplete="off"
              spellCheck={false}
              className="font-mono"
              {...form.register("id", {
                onChange: (change: { target: { value: string } }) => setIdFollowsTitle(automation === null && change.target.value.trim() === ""),
              })}
            />
          </FormField>
          <div className="grid gap-2 sm:pt-0.5">
            <Label htmlFor="automation-enabled">{t("automationBuilder.enabled")}</Label>
            <Controller
              control={form.control}
              name="enabled"
              render={({ field }) => <Switch id="automation-enabled" checked={field.value} onCheckedChange={field.onChange} />}
            />
          </div>
        </div>
        <div className="mt-4">
          <TagsField id="automation-tags" control={form.control} name="tags" suggestions={catalogue.choices.tags} error={errors.tags?.message ?? (Array.isArray(errors.tags) ? errors.tags.find(Boolean)?.message : undefined)} />
        </div>
      </SectionCard>
      <div className="grid items-start gap-6 lg:grid-cols-2">
        <WhenCard form={form} catalogue={catalogue} />
        <FiltersCard form={form} catalogue={catalogue} />
      </div>
      <RunCard form={form as unknown as UseFormReturn<RunFields>} event={event} scripts={scripts} />
      <CommandPreview event={event} script={values.script} args={values.args.map((argument) => argument.value)} chosen={chosen} />
      <div className="glass-panel sticky bottom-3 z-20 flex justify-end gap-2 rounded-xl px-4 py-3">
        <Button asChild variant="outline">
          <Link href={routes.adminAutomations}>{t("common.cancel")}</Link>
        </Button>
        <Button type="submit" disabled={form.formState.isSubmitting || errors.args !== undefined}>
          {form.formState.isSubmitting ? t("common.saving") : t("common.save")}
        </Button>
      </div>
    </form>
  );
}
