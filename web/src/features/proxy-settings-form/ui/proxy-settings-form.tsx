"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import { useTranslations } from "next-intl";
import { useEffect, useState } from "react";
import { Controller, type Path, useForm, useWatch } from "react-hook-form";
import { toast } from "sonner";

import {
  PROXY_TLS_MODES,
  type Proxy,
  type ProxySettingsForm as ProxySettingsFormValues,
  proxySettingsFormOf,
  proxySettingsFormSchema,
  useSaveProxySettings,
} from "@/entities/proxy";
import { ConflictError, ValidationError } from "@/shared/api";
import { ErrorNotice } from "@/shared/ui/error-notice";
import { FormField } from "@/shared/ui/form-field";
import { Button, Input, Label, Switch } from "@/shared/ui/primitives";

const SELECT =
  "h-9 w-full rounded-md border border-input bg-glass-tint px-3 text-sm shadow-xs outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50";

const FIELDS: Record<string, Path<ProxySettingsFormValues>> = {
  "proxy.enabled": "enabled",
  "proxy.http_port": "http_port",
  "proxy.https_port": "https_port",
  "proxy.portal_host": "portal_host",
  "proxy.cookie_domain": "cookie_domain",
  "proxy.tls.mode": "mode",
  "proxy.tls.email": "email",
  "proxy.tls.certificate": "certificate",
  "proxy.tls.key": "key",
};

export type ProxySettingsFormProps = { proxy: Proxy; revision: string | null };

export function ProxySettingsForm({ proxy, revision }: ProxySettingsFormProps) {
  const t = useTranslations();
  const save = useSaveProxySettings();
  const [problems, setProblems] = useState<string[]>([]);
  const [conflict, setConflict] = useState(false);
  const form = useForm<ProxySettingsFormValues>({ resolver: zodResolver(proxySettingsFormSchema), defaultValues: proxySettingsFormOf(proxy) });
  const { reset, formState } = form;

  useEffect(() => {
    if (!formState.isDirty) {
      reset(proxySettingsFormOf(proxy));
    }
  }, [proxy, formState.isDirty, reset]);

  const submit = form.handleSubmit(async (values) => {
    setProblems([]);
    setConflict(false);
    try {
      const saved = await save.mutateAsync({ form: values, revision });
      reset(proxySettingsFormOf(saved.data));
      toast.success(t(values.enabled ? "proxy.settings.enabled" : "proxy.settings.disabled"));
    } catch (error) {
      if (error instanceof ValidationError) {
        const elsewhere: string[] = [];
        for (const { field, message } of error.fields) {
          const path = FIELDS[field];
          if (path) {
            form.setError(path, { message });
          } else {
            elsewhere.push(`${field}: ${message}`);
          }
        }
        setProblems(elsewhere);
      } else if (error instanceof ConflictError) {
        setConflict(true);
      } else {
        toast.error(t("errors.generic"));
      }
    }
  });

  const errors = formState.errors;
  const mode = useWatch({ control: form.control, name: "mode" });
  return (
    <form onSubmit={submit} className="grid gap-5" noValidate>
      {conflict ? <ErrorNotice title={t("errors.conflict")} /> : null}
      {problems.length > 0 ? <ErrorNotice title={t("proxy.settings.refused")} description={problems.join("; ")} /> : null}
      <div className="flex items-center justify-between gap-4">
        <Label htmlFor="proxy-enabled">{t("proxy.settings.enabledLabel")}</Label>
        <Controller
          control={form.control}
          name="enabled"
          render={({ field }) => <Switch id="proxy-enabled" checked={field.value} onCheckedChange={field.onChange} />}
        />
      </div>
      <div className="grid gap-5 sm:grid-cols-2">
        <FormField id="proxy-portal-host" label={t("proxy.settings.portalHost")} hint={t("proxy.settings.portalHostHint")} error={errors.portal_host?.message}>
          <Input id="proxy-portal-host" spellCheck={false} autoComplete="off" {...form.register("portal_host")} />
        </FormField>
        <FormField
          id="proxy-cookie-domain"
          label={t("proxy.settings.cookieDomain")}
          hint={t("proxy.settings.cookieDomainHint")}
          optional
          error={errors.cookie_domain?.message}
        >
          <Input id="proxy-cookie-domain" spellCheck={false} autoComplete="off" {...form.register("cookie_domain")} />
        </FormField>
      </div>
      <div className="grid gap-5 sm:grid-cols-2">
        <FormField id="proxy-https-port" label={t("proxy.settings.httpsPort")} hint={t("proxy.settings.httpsPortHint")} error={errors.https_port?.message}>
          <Input id="proxy-https-port" type="number" inputMode="numeric" {...form.register("https_port", { valueAsNumber: true })} />
        </FormField>
        <FormField id="proxy-http-port" label={t("proxy.settings.httpPort")} hint={t("proxy.settings.httpPortHint")} error={errors.http_port?.message}>
          <Input id="proxy-http-port" type="number" inputMode="numeric" {...form.register("http_port", { valueAsNumber: true })} />
        </FormField>
      </div>
      <FormField id="proxy-tls" label={t("proxy.settings.tls")} hint={t(`serviceForm.publicationTlsHints.${mode}`)}>
        <select id="proxy-tls" className={SELECT} {...form.register("mode")}>
          {PROXY_TLS_MODES.map((value) => (
            <option key={value} value={value}>
              {t(`proxy.tlsModes.${value}`)}
            </option>
          ))}
        </select>
      </FormField>
      {mode === "acme" ? (
        <FormField id="proxy-email" label={t("serviceForm.publicationEmail")} optional error={errors.email?.message}>
          <Input id="proxy-email" type="email" {...form.register("email")} />
        </FormField>
      ) : null}
      {mode === "files" ? (
        <div className="grid gap-5 sm:grid-cols-2">
          <FormField id="proxy-certificate" label={t("serviceForm.publicationCertificate")} error={errors.certificate?.message}>
            <Input id="proxy-certificate" spellCheck={false} {...form.register("certificate")} />
          </FormField>
          <FormField id="proxy-key" label={t("serviceForm.publicationKey")} error={errors.key?.message}>
            <Input id="proxy-key" spellCheck={false} {...form.register("key")} />
          </FormField>
        </div>
      ) : null}
      <p className="text-xs text-muted-foreground">{t("proxy.settings.loopbackHint")}</p>
      <div className="flex justify-end">
        <Button type="submit" disabled={formState.isSubmitting || !formState.isDirty}>
          {formState.isSubmitting ? t("common.saving") : t("common.save")}
        </Button>
      </div>
    </form>
  );
}
