"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import { useTranslations } from "next-intl";
import { useEffect, useState } from "react";
import { type Path, useForm } from "react-hook-form";
import { toast } from "sonner";

import { type NetworkForm as NetworkFormValues, type NetworkSettings, networkFormOf, networkFormSchema, useSaveNetwork } from "@/entities/network";
import { ConflictError, ValidationError } from "@/shared/api";
import { ErrorNotice } from "@/shared/ui/error-notice";
import { FormField } from "@/shared/ui/form-field";
import { Button, Input } from "@/shared/ui/primitives";

import { byField } from "../model/server-errors";

export type NetworkFormProps = { configured: NetworkSettings; revision: string | null };

export function NetworkForm({ configured, revision }: NetworkFormProps) {
  const t = useTranslations();
  const save = useSaveNetwork();
  const [conflict, setConflict] = useState(false);
  const form = useForm<NetworkFormValues>({ resolver: zodResolver(networkFormSchema), defaultValues: networkFormOf(configured) });
  const { reset, formState } = form;

  useEffect(() => {
    if (!formState.isDirty) {
      reset(networkFormOf(configured));
    }
  }, [configured, formState.isDirty, reset]);

  const submit = form.handleSubmit(async (values) => {
    setConflict(false);
    try {
      const saved = await save.mutateAsync({ form: values, revision });
      reset(networkFormOf(saved.data.configured));
      toast.success(t("network.saved"));
    } catch (error) {
      if (error instanceof ValidationError) {
        for (const { path, message } of byField(error.fields)) {
          form.setError(path as Path<NetworkFormValues>, { message });
        }
      } else if (error instanceof ConflictError) {
        setConflict(true);
      } else {
        toast.error(t("errors.generic"));
      }
    }
  });

  const errors = formState.errors;
  return (
    <form onSubmit={submit} className="grid gap-5" noValidate>
      {conflict ? <ErrorNotice title={t("errors.conflict")} /> : null}
      <div className="grid gap-5 sm:grid-cols-[1fr_10rem]">
        <FormField id="network-address" label={t("network.address")} hint={t("network.addressHint")} error={errors.address?.message}>
          <Input id="network-address" spellCheck={false} {...form.register("address")} />
        </FormField>
        <FormField id="network-port" label={t("network.port")} error={errors.port?.message}>
          <Input id="network-port" type="number" inputMode="numeric" {...form.register("port", { valueAsNumber: true })} />
        </FormField>
      </div>
      <FormField id="network-public-url" label={t("network.publicUrl")} hint={t("network.publicUrlHint")} optional error={errors.public_url?.message}>
        <Input id="network-public-url" type="url" spellCheck={false} {...form.register("public_url")} />
      </FormField>
      <FormField
        id="network-trusted-proxies"
        label={t("network.trustedProxies")}
        hint={t("network.trustedProxiesHint")}
        optional
        error={errors.trusted_proxies?.message}
      >
        <textarea
          id="network-trusted-proxies"
          rows={3}
          spellCheck={false}
          className="min-h-20 w-full rounded-md border border-input bg-transparent px-3 py-2 font-mono text-sm shadow-xs outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50"
          {...form.register("trusted_proxies")}
        />
      </FormField>
      <div className="flex justify-end">
        <Button type="submit" disabled={formState.isSubmitting || !formState.isDirty}>
          {formState.isSubmitting ? t("common.saving") : t("common.save")}
        </Button>
      </div>
    </form>
  );
}
