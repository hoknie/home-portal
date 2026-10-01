"use client";

import { useTranslations } from "next-intl";
import { Controller, type UseFormReturn } from "react-hook-form";

import { PUBLICATION_TLS, type ServiceForm, mainAddressOf } from "@/entities/service";
import { FormField } from "@/shared/ui/form-field";
import { Input, Label, Switch } from "@/shared/ui/primitives";

const SELECT =
  "h-9 w-full rounded-md border border-input bg-glass-tint px-3 text-sm shadow-xs outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50";

export function ProxySettings({ form }: { form: UseFormReturn<ServiceForm> }) {
  const t = useTranslations();
  const errors = form.formState.errors.proxy;
  const tls = form.watch("proxy.tls");
  const https = mainAddressOf({ rows: form.watch("rows") }).trim().startsWith("https://");
  const message = (text: string | undefined) => (text && t.has(text as Parameters<typeof t.has>[0]) ? t(text as Parameters<typeof t>[0]) : text);
  return (
    <details className="group rounded-lg border border-glass-edge bg-glass-tint px-4 py-3">
      <summary className="cursor-pointer text-sm font-medium">{t("serviceForm.rows.proxySettings")}</summary>
      <div className="mt-4 grid gap-4">
        <div className="grid gap-4 sm:grid-cols-2">
          <FormField id="proxy-tls" label={t("serviceForm.publicationTls")} hint={t(`serviceForm.publicationTlsHints.${tls}`)}>
            <select id="proxy-tls" className={SELECT} {...form.register("proxy.tls")}>
              {PUBLICATION_TLS.map((mode) => (
                <option key={mode} value={mode}>
                  {t(`serviceForm.publicationTlsModes.${mode}`)}
                </option>
              ))}
            </select>
          </FormField>
          {tls === "acme" ? (
            <FormField id="proxy-email" label={t("serviceForm.publicationEmail")} optional error={message(errors?.email?.message)}>
              <Input id="proxy-email" type="email" {...form.register("proxy.email")} />
            </FormField>
          ) : null}
        </div>
        {tls === "files" ? (
          <div className="grid gap-4 sm:grid-cols-2">
            <FormField id="proxy-certificate" label={t("serviceForm.publicationCertificate")} error={message(errors?.certificate?.message)}>
              <Input id="proxy-certificate" spellCheck={false} {...form.register("proxy.certificate")} />
            </FormField>
            <FormField id="proxy-key" label={t("serviceForm.publicationKey")} error={message(errors?.key?.message)}>
              <Input id="proxy-key" spellCheck={false} {...form.register("proxy.key")} />
            </FormField>
          </div>
        ) : null}
        {https ? (
          <div className="flex items-center justify-between gap-4">
            <Label htmlFor="proxy-verify">{t("serviceForm.publicationVerify")}</Label>
            <Controller
              control={form.control}
              name="proxy.upstream_verify"
              render={({ field }) => <Switch id="proxy-verify" checked={field.value} onCheckedChange={field.onChange} />}
            />
          </div>
        ) : null}
      </div>
    </details>
  );
}
