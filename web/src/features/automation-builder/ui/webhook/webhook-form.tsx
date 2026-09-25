"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import Link from "next/link";
import { useTranslations } from "next-intl";
import { useMemo, useState } from "react";
import { Controller, type Path, type UseFormReturn, useForm, useWatch } from "react-hook-form";
import { toast } from "sonner";

import type { Catalogue, Scripts } from "@/entities/automation";
import { type Webhook, useSaveWebhook } from "@/entities/webhook";
import { ConflictError, ValidationError } from "@/shared/api";
import { routes } from "@/shared/config";
import { useLeaveGuard } from "@/shared/lib/leave-guard";
import { ErrorNotice } from "@/shared/ui/error-notice";
import { FormField } from "@/shared/ui/form-field";
import { Input, Label, Switch, Button } from "@/shared/ui/primitives";
import { SectionCard } from "@/shared/ui/section-card";
import { TagInput } from "@/shared/ui/tag-input";

import type { RunFields } from "../../model/run-fields";
import { byField } from "../../model/server-errors";
import { type WebhookForm as WebhookFormValues, emptyWebhookForm, webhookEventOf, webhookFormOf, webhookFormSchema, webhookRequestOf } from "../../model/webhook-form";
import { CommandPreview } from "../command-preview";
import { RunCard } from "../run-card";
import { TokenActions } from "./token-actions";
import { TagsField } from "../tags-field";
import { TokenDialog } from "./token-dialog";

export type WebhookFormProps = {
  webhook: Webhook | null;
  revision: string | null;
  catalogue: Catalogue;
  scripts: Scripts;
  onSaved: () => void;
  onConflict: () => void;
};

export function WebhookForm({ webhook, revision, catalogue, scripts, onSaved, onConflict }: WebhookFormProps) {
  const t = useTranslations();
  const save = useSaveWebhook();
  const [conflict, setConflict] = useState(false);
  const [saved, setSaved] = useState(false);
  const [issued, setIssued] = useState<{ token: string; address: string } | null>(null);
  const schema = useMemo(() => webhookFormSchema((variables) => webhookEventOf(catalogue, variables).fields.map((field) => field.name)), [catalogue]);
  const form = useForm<WebhookFormValues>({ resolver: zodResolver(schema), mode: "onChange", defaultValues: webhook ? webhookFormOf(webhook) : emptyWebhookForm });
  useLeaveGuard(form.formState.isDirty && !saved, t("automationBuilder.leave"));
  const values = useWatch({ control: form.control }) as WebhookFormValues;
  const event = webhookEventOf(catalogue, values.variables);
  const errors = form.formState.errors;

  const submit = form.handleSubmit(async (current) => {
    setConflict(false);
    try {
      const created = await save.mutateAsync({ id: webhook?.id ?? null, body: webhookRequestOf(current, webhook === null), revision });
      setSaved(true);
      toast.success(t(webhook ? "webhooks.saved" : "webhooks.created"));
      if (created) {
        setIssued(created);
      } else {
        onSaved();
      }
    } catch (error) {
      if (error instanceof ValidationError) {
        for (const { path, message } of byField(error.fields)) {
          form.setError(path as Path<WebhookFormValues>, { message });
        }
      } else if (error instanceof ConflictError) {
        setConflict(true);
        onConflict();
      } else {
        toast.error(t("errors.generic"));
      }
    }
  });

  return (
    <form onSubmit={submit} className="grid gap-6" noValidate>
      {conflict ? <ErrorNotice title={t("errors.conflict")} /> : null}
      <SectionCard title={t("webhooks.identity")} description={t("webhooks.identityDescription")}>
        <div className="grid gap-4">
          <div className="grid gap-4 sm:grid-cols-[1fr_auto] sm:items-start">
            <FormField id="webhook-title" label={t("webhooks.titleLabel")} error={errors.title?.message}>
              <Input id="webhook-title" autoFocus {...form.register("title")} />
            </FormField>
            <div className="grid gap-2">
              <Label htmlFor="webhook-enabled">{t("webhooks.enabledLabel")}</Label>
              <Controller control={form.control} name="enabled" render={({ field }) => <Switch id="webhook-enabled" checked={field.value} onCheckedChange={field.onChange} />} />
            </div>
          </div>
          <TagsField id="webhook-tags" control={form.control} name="tags" suggestions={catalogue.choices.tags} error={errors.tags?.message ?? (Array.isArray(errors.tags) ? errors.tags.find(Boolean)?.message : undefined)} />
          <FormField id="webhook-variables" label={t("webhooks.variables")} hint={t("webhooks.variablesHint")} optional error={errors.variables?.message}>
            <Controller
              control={form.control}
              name="variables"
              render={({ field }) => (
                <TagInput
                  id="webhook-variables"
                  values={field.value}
                  onChange={field.onChange}
                  onBlur={field.onBlur}
                  suggestions={[]}
                  removeLabel={(value) => t("tagInput.remove", { value })}
                  createLabel={(value) => t("tagInput.create", { value })}
                />
              )}
            />
          </FormField>
          <div role="radiogroup" aria-labelledby="webhook-action-label" className="grid gap-2">
            <Label asChild>
              <span id="webhook-action-label">{t("webhooks.action")}</span>
            </Label>
            {(["script", "event"] as const).map((action) => (
              <label key={action} className="flex items-start gap-2 text-sm">
                <input type="radio" className="mt-1 accent-primary" value={action} {...form.register("action")} />
                <span className="grid gap-0.5">
                  <span className="font-medium">{t(`webhooks.actions.${action}`)}</span>
                  <span className="text-xs text-muted-foreground">{t(`webhooks.actionHints.${action}`)}</span>
                </span>
              </label>
            ))}
          </div>
          {webhook ? (
            <TokenActions webhook={webhook} revision={revision} />
          ) : (
            <div className="flex items-start justify-between gap-4">
              <div className="grid gap-1">
                <Label htmlFor="webhook-token">{t("webhooks.withToken")}</Label>
                <p className="text-xs text-muted-foreground">{t("webhooks.withTokenHint")}</p>
              </div>
              <Controller control={form.control} name="with_token" render={({ field }) => <Switch id="webhook-token" checked={field.value} onCheckedChange={field.onChange} />} />
            </div>
          )}
        </div>
      </SectionCard>
      {values.action === "script" ? (
        <>
          <RunCard form={form as unknown as UseFormReturn<RunFields>} event={event} scripts={scripts} />
          <CommandPreview event={event} script={values.script} args={values.args.map((argument) => argument.value)} chosen={{ "webhook.id": webhook?.id, "webhook.title": values.title || undefined }} />
        </>
      ) : null}
      <div className="glass-panel sticky bottom-3 z-20 flex justify-end gap-2 rounded-xl px-4 py-3">
        <Button asChild variant="outline">
          <Link href={routes.adminWebhooks}>{t("common.cancel")}</Link>
        </Button>
        <Button type="submit" disabled={form.formState.isSubmitting || errors.args !== undefined}>
          {form.formState.isSubmitting ? t("common.saving") : t("common.save")}
        </Button>
      </div>
      <TokenDialog
        token={issued?.token ?? null}
        address={issued?.address ?? webhook?.address ?? ""}
        variables={values.variables}
        onClose={() => {
          setIssued(null);
          onSaved();
        }}
      />
    </form>
  );
}
