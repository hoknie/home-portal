"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import { useTranslations } from "next-intl";
import { useEffect, useState } from "react";
import { Controller, type Path, useForm } from "react-hook-form";
import { toast } from "sonner";

import { type Dns, type DnsForm, dnsFormOf, dnsFormSchema, useSaveDns } from "@/entities/dns-server";
import { ConflictError, ValidationError } from "@/shared/api";
import { ErrorNotice } from "@/shared/ui/error-notice";
import { FormField } from "@/shared/ui/form-field";
import { Button, Input, Label, Switch } from "@/shared/ui/primitives";
import { TagInput } from "@/shared/ui/tag-input";

const FIELDS: Record<string, Path<DnsForm>> = {
  "dns.enabled": "enabled",
  "dns.address": "address",
  "dns.port": "port",
  "dns.ttl": "ttl",
  "dns.tls.port": "tls_port",
  "dns.tls.enabled": "tls_enabled",
  "dns.https.enabled": "https_enabled",
  "dns.https.host": "https_host",
};

export function fieldOf(server: string): Path<DnsForm> | null {
  if (FIELDS[server]) {
    return FIELDS[server];
  }
  if (server.startsWith("dns.zones")) {
    return "zones";
  }
  const address = /^dns\.addresses\.([a-z0-9-]+)$/.exec(server);
  return address ? (`addresses.${address[1]}` as Path<DnsForm>) : null;
}

export type DnsSettingsFormProps = { dns: Dns; revision: string | null };

export function DnsSettingsForm({ dns, revision }: DnsSettingsFormProps) {
  const t = useTranslations("dns");
  const common = useTranslations();
  const save = useSaveDns();
  const [problems, setProblems] = useState<string[]>([]);
  const [conflict, setConflict] = useState(false);
  const form = useForm<DnsForm>({ resolver: zodResolver(dnsFormSchema), defaultValues: dnsFormOf(dns) });
  const { reset, formState } = form;

  useEffect(() => {
    if (!formState.isDirty) {
      reset(dnsFormOf(dns));
    }
  }, [dns, formState.isDirty, reset]);

  const submit = form.handleSubmit(async (values) => {
    setProblems([]);
    setConflict(false);
    try {
      const saved = await save.mutateAsync({ form: values, dns, revision });
      reset(dnsFormOf(saved.data));
      toast.success(t("saved"));
    } catch (error) {
      if (error instanceof ValidationError) {
        const elsewhere: string[] = [];
        for (const { field, message } of error.fields) {
          const path = fieldOf(field);
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
        toast.error(common("errors.generic"));
      }
    }
  });

  const errors = formState.errors;
  return (
    <form onSubmit={submit} className="grid gap-5" noValidate>
      {conflict ? <ErrorNotice title={common("errors.conflict")} /> : null}
      {problems.length > 0 ? <ErrorNotice title={t("refused")} description={problems.join("; ")} /> : null}
      <div className="flex items-center justify-between gap-4">
        <Label htmlFor="dns-enabled">{t("enabledLabel")}</Label>
        <Controller control={form.control} name="enabled" render={({ field }) => <Switch id="dns-enabled" checked={field.value} onCheckedChange={field.onChange} />} />
      </div>
      <div className="grid gap-5 sm:grid-cols-3">
        <FormField id="dns-address" label={t("address")} hint={t("addressHint")} error={errors.address?.message}>
          <Input id="dns-address" spellCheck={false} autoComplete="off" {...form.register("address")} />
        </FormField>
        <FormField id="dns-port" label={t("port")} hint={t("portHint")} error={errors.port?.message}>
          <Input id="dns-port" type="number" inputMode="numeric" {...form.register("port", { valueAsNumber: true })} />
        </FormField>
        <FormField id="dns-ttl" label={t("ttl")} hint={t("ttlHint")} error={errors.ttl?.message}>
          <Input id="dns-ttl" type="number" inputMode="numeric" {...form.register("ttl", { valueAsNumber: true })} />
        </FormField>
      </div>
      <FormField id="dns-zones" label={t("zones")} hint={t("zonesHint")} error={errors.zones?.message}>
        <Controller
          control={form.control}
          name="zones"
          render={({ field }) => (
            <TagInput
              id="dns-zones"
              values={field.value}
              onChange={field.onChange}
              suggestions={[]}
              invalid={errors.zones !== undefined}
              removeLabel={(value) => t("removeZone", { zone: value })}
              createLabel={(value) => t("addZone", { zone: value })}
            />
          )}
        />
      </FormField>
      <fieldset className="grid gap-3">
        <legend className="text-sm font-medium">{t("addresses")}</legend>
        <p className="text-xs text-muted-foreground">{t("addressesHint")}</p>
        {dns.environments.map((environment) => (
          <FormField
            key={environment}
            id={`dns-address-${environment}`}
            label={environment}
            optional
            error={(errors.addresses as Record<string, { message?: string }> | undefined)?.[environment]?.message}
          >
            <Input
              id={`dns-address-${environment}`}
              spellCheck={false}
              autoComplete="off"
              placeholder={t("automatic")}
              {...form.register(`addresses.${environment}` as Path<DnsForm>)}
            />
          </FormField>
        ))}
      </fieldset>
      <div className="grid gap-5 sm:grid-cols-2">
        <div className="grid gap-3">
          <div className="flex items-center justify-between gap-4">
            <Label htmlFor="dns-tls">{t("tlsLabel")}</Label>
            <Controller control={form.control} name="tls_enabled" render={({ field }) => <Switch id="dns-tls" checked={field.value} onCheckedChange={field.onChange} />} />
          </div>
          <FormField id="dns-tls-port" label={t("tlsPort")} error={errors.tls_port?.message}>
            <Input id="dns-tls-port" type="number" inputMode="numeric" {...form.register("tls_port", { valueAsNumber: true })} />
          </FormField>
        </div>
        <div className="grid gap-3">
          <div className="flex items-center justify-between gap-4">
            <Label htmlFor="dns-https">{t("httpsLabel")}</Label>
            <Controller control={form.control} name="https_enabled" render={({ field }) => <Switch id="dns-https" checked={field.value} onCheckedChange={field.onChange} />} />
          </div>
          <FormField id="dns-https-host" label={t("httpsHost")} hint={t("httpsHostHint")} optional error={errors.https_host?.message ?? errors.https_enabled?.message}>
            <Input id="dns-https-host" spellCheck={false} autoComplete="off" {...form.register("https_host")} />
          </FormField>
        </div>
      </div>
      <div>
        <Button type="submit" disabled={save.isPending}>
          {t("save")}
        </Button>
      </div>
    </form>
  );
}
