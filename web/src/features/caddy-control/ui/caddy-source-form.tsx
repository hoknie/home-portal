"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import { useTranslations } from "next-intl";
import { useEffect, useState } from "react";
import { type Path, useForm } from "react-hook-form";
import { toast } from "sonner";

import { type CaddySourceForm as CaddySourceFormValues, type Proxy, caddySourceFormOf, caddySourceFormSchema, useSaveCaddySource } from "@/entities/proxy";
import { ConflictError, ValidationError } from "@/shared/api";
import { ErrorNotice } from "@/shared/ui/error-notice";
import { FormField } from "@/shared/ui/form-field";
import { Button, Input } from "@/shared/ui/primitives";

const FIELDS: Record<string, Path<CaddySourceFormValues>> = {
  "proxy.caddy.source": "source",
  "proxy.caddy.version": "version",
};

export type CaddySourceFormProps = { proxy: Proxy; revision: string | null };

export function CaddySourceForm({ proxy, revision }: CaddySourceFormProps) {
  const t = useTranslations();
  const save = useSaveCaddySource();
  const [conflict, setConflict] = useState(false);
  const form = useForm<CaddySourceFormValues>({ resolver: zodResolver(caddySourceFormSchema), defaultValues: caddySourceFormOf(proxy) });
  const { reset, formState } = form;

  useEffect(() => {
    if (!formState.isDirty) {
      reset(caddySourceFormOf(proxy));
    }
  }, [proxy, formState.isDirty, reset]);

  const submit = form.handleSubmit(async (values) => {
    setConflict(false);
    try {
      const saved = await save.mutateAsync({ form: values, revision });
      reset(caddySourceFormOf(saved.data));
      toast.success(t("proxy.caddyControl.sourceSaved"));
    } catch (error) {
      if (error instanceof ValidationError) {
        for (const { field, message } of error.fields) {
          const path = FIELDS[field];
          if (path) {
            form.setError(path, { message });
          } else {
            toast.error(`${field}: ${message}`);
          }
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
    <form onSubmit={submit} className="grid gap-4" noValidate aria-label={t("proxy.caddyControl.sourceTitle")}>
      {conflict ? <ErrorNotice title={t("errors.conflict")} /> : null}
      <div className="grid gap-4 sm:grid-cols-[2fr_1fr]">
        <FormField id="caddy-source" label={t("proxy.caddyControl.source")} hint={t("proxy.caddyControl.sourceHint")} error={errors.source?.message}>
          <Input id="caddy-source" type="url" inputMode="url" spellCheck={false} autoComplete="off" {...form.register("source")} />
        </FormField>
        <FormField id="caddy-version" label={t("proxy.caddyControl.versionField")} hint={t("proxy.caddyControl.versionHint")} error={errors.version?.message}>
          <Input id="caddy-version" spellCheck={false} autoComplete="off" {...form.register("version")} />
        </FormField>
      </div>
      <div className="flex justify-end">
        <Button type="submit" variant="outline" size="sm" disabled={formState.isSubmitting || !formState.isDirty}>
          {formState.isSubmitting ? t("common.saving") : t("proxy.caddyControl.sourceSave")}
        </Button>
      </div>
    </form>
  );
}
