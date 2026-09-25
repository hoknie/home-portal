"use client";

import { LoaderCircle } from "lucide-react";
import { useTranslations } from "next-intl";
import type { UseFormReturn } from "react-hook-form";

import { type ServiceForm, ServiceIcon } from "@/entities/service";
import { FormField } from "@/shared/ui/form-field";
import { Input } from "@/shared/ui/primitives";

import { useIconPreview } from "../model/use-icon-preview";

export function IconField({ form }: { form: UseFormReturn<ServiceForm> }) {
  const t = useTranslations();
  const preview = useIconPreview(form.watch("icon"), form.watch("url"));
  const error = form.formState.errors.icon?.message ?? preview.problem ?? undefined;
  return (
    <FormField id="service-icon" label={t("serviceForm.icon")} hint={t("serviceForm.iconHint")} optional error={error}>
      <div className="flex items-center gap-3">
        <span
          className="flex size-9 shrink-0 items-center justify-center rounded-lg border border-glass-edge bg-glass-tint"
          data-icon-preview={preview.source ? "image" : (preview.name ?? "default")}
          aria-busy={preview.loading || undefined}
          title={t("serviceForm.iconPreview")}
        >
          {preview.loading ? (
            <LoaderCircle className="size-4 animate-spin text-muted-foreground" aria-label={t("serviceForm.iconLoading")} />
          ) : (
            <ServiceIcon key={preview.source ?? "none"} name={preview.name} source={preview.source} />
          )}
        </span>
        <Input id="service-icon" spellCheck={false} {...form.register("icon", { onBlur: preview.flush })} />
      </div>
    </FormField>
  );
}
