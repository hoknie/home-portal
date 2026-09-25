"use client";

import { Plus, X } from "lucide-react";
import { useTranslations } from "next-intl";
import { type UseFormReturn, useFieldArray } from "react-hook-form";

import type { ServiceForm } from "@/entities/service";
import { FormField } from "@/shared/ui/form-field";
import { Button, Input, Label } from "@/shared/ui/primitives";

const TEXTAREA =
  "min-h-28 w-full rounded-md border border-input bg-glass-tint px-3 py-2 text-sm shadow-xs outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50";

export function DetailsFields({ form }: { form: UseFormReturn<ServiceForm> }) {
  const t = useTranslations();
  const links = useFieldArray({ control: form.control, name: "links" });
  const errors = form.formState.errors;
  return (
    <div className="grid gap-4">
      <fieldset className="grid gap-2">
        <Label asChild>
          <legend>{t("serviceForm.links")}</legend>
        </Label>
        {links.fields.map((field, index) => (
          <div key={field.id} className="grid grid-cols-[1fr_1.5fr_auto] items-start gap-2">
            <FormField id={`link-title-${index}`} label={t("serviceForm.linkTitle")} error={errors.links?.[index]?.title?.message}>
              <Input id={`link-title-${index}`} {...form.register(`links.${index}.title`)} />
            </FormField>
            <FormField id={`link-url-${index}`} label={t("serviceForm.linkUrl")} error={errors.links?.[index]?.url?.message}>
              <Input id={`link-url-${index}`} type="url" inputMode="url" spellCheck={false} {...form.register(`links.${index}.url`)} />
            </FormField>
            <Button type="button" variant="ghost" size="icon" className="mt-6" aria-label={t("serviceForm.removeLink")} onClick={() => links.remove(index)}>
              <X aria-hidden />
            </Button>
          </div>
        ))}
        <Button type="button" variant="outline" size="sm" className="justify-self-start" onClick={() => links.append({ title: "", url: "" })}>
          <Plus aria-hidden />
          {t("serviceForm.addLink")}
        </Button>
      </fieldset>
      <FormField id="service-notes" label={t("serviceForm.notes")} hint={t("serviceForm.notesHint")} optional error={errors.notes?.message}>
        <textarea id="service-notes" className={TEXTAREA} {...form.register("notes")} />
      </FormField>
    </div>
  );
}
