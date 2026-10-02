"use client";

import { Plus, X } from "lucide-react";
import { useTranslations } from "next-intl";
import { type UseFormReturn, useFieldArray } from "react-hook-form";

import type { ServiceForm } from "@/entities/service";
import { Button, FormField, Input, Label, Textarea } from "@/shared/ui/kit";


const ROW = "grid grid-cols-[minmax(0,1fr)_minmax(0,1.5fr)_2.25rem] items-start gap-2";

function Message({ text }: { text: string | undefined }) {
  const t = useTranslations();
  if (!text) {
    return null;
  }
  return (
    <p role="alert" className="text-sm text-destructive">
      {t.has(text as Parameters<typeof t.has>[0]) ? t(text as Parameters<typeof t>[0]) : text}
    </p>
  );
}

export function DetailsFields({ form }: { form: UseFormReturn<ServiceForm> }) {
  const t = useTranslations();
  const links = useFieldArray({ control: form.control, name: "links" });
  const errors = form.formState.errors;
  return (
    <div className="grid gap-4">
      <fieldset className="grid gap-2">
        <Label asChild className="mb-3">
          <legend>{t("serviceForm.links")}</legend>
        </Label>
        {links.fields.length > 0 ? (
          <div className={`${ROW} text-xs font-medium text-muted-foreground`} aria-hidden>
            <span>{t("serviceForm.linkTitleHeading")}</span>
            <span>{t("serviceForm.linkUrlHeading")}</span>
            <span />
          </div>
        ) : null}
        {links.fields.map((field, index) => (
          <div key={field.id} className={ROW}>
            <div className="grid gap-1">
              <label className="sr-only" htmlFor={`link-title-${index}`}>
                {t("serviceForm.linkTitle")}
              </label>
              <Input id={`link-title-${index}`} aria-invalid={errors.links?.[index]?.title ? true : undefined} {...form.register(`links.${index}.title`)} />
              <Message text={errors.links?.[index]?.title?.message} />
            </div>
            <div className="grid gap-1">
              <label className="sr-only" htmlFor={`link-url-${index}`}>
                {t("serviceForm.linkUrl")}
              </label>
              <Input id={`link-url-${index}`} type="url" inputMode="url" spellCheck={false} aria-invalid={errors.links?.[index]?.url ? true : undefined} {...form.register(`links.${index}.url`)} />
              <Message text={errors.links?.[index]?.url?.message} />
            </div>
            <Button type="button" variant="ghost" size="icon" aria-label={t("serviceForm.removeLink")} onClick={() => links.remove(index)}>
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
        <Textarea id="service-notes" {...form.register("notes")} />
      </FormField>
    </div>
  );
}
