"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import Link from "next/link";
import { useTranslations } from "next-intl";
import { useState } from "react";
import { Controller, type Path, useForm } from "react-hook-form";
import { toast } from "sonner";

import { useCan } from "@/entities/session";
import { type Service, type ServiceForm as ServiceFormValues, emptyServiceForm, formOf, serviceFormSchema, useSaveService } from "@/entities/service";
import { ConflictError, type FieldError, ValidationError } from "@/shared/api";
import { useEditorRevision } from "@/shared/lib/editor-revision";
import { useLeaveGuard } from "@/shared/lib/leave-guard";
import { ConflictNotice } from "@/shared/ui/conflict-notice";
import { ErrorNotice } from "@/shared/ui/error-notice";
import { FormField } from "@/shared/ui/form-field";
import { Button, Input } from "@/shared/ui/primitives";
import { SectionCard } from "@/shared/ui/section-card";
import { TagInput } from "@/shared/ui/tag-input";

import { byField } from "../model/server-errors";
import { slugOf, uniqueId } from "@/shared/lib/slug";
import { AddressRows } from "./address-rows";
import { DetailsFields } from "./details-fields";
import { IconField } from "./icon-field";
import { ProbeFields } from "./probe-fields";


export type ServiceFormProps = {
  service: Service | null;
  environments: string[];
  cancelHref: string;
  revision: string | null;
  taken: string[];
  groups: string[];
  onSaved: () => void;
  onConflict: () => void;
};

function initialOf(service: Service | null, environments: string[]) {
  return service ? formOf(service, environments) : emptyServiceForm(environments);
}

export function ServiceForm({ service, environments, cancelHref, revision, taken, groups, onSaved, onConflict }: ServiceFormProps) {
  const t = useTranslations();
  const can = useCan();
  const editable = can("services", service === null ? "create" : "update");
  const save = useSaveService();
  const [conflict, setConflict] = useState(false);
  const [unplaced, setUnplaced] = useState<FieldError[]>([]);
  const [saved, setSaved] = useState(false);
  const [idFollowsName, setIdFollowsName] = useState(service === null);
  const form = useForm<ServiceFormValues>({ resolver: zodResolver(serviceFormSchema), defaultValues: initialOf(service, environments) });
  useLeaveGuard(form.formState.isDirty && !saved, t("serviceForm.leave"));

  const held = useEditorRevision(revision);
  const send = (at: string | null) => form.handleSubmit(async (values) => {
    setConflict(false);
    setUnplaced([]);
    try {
      const stored = await save.mutateAsync({ id: service?.id ?? null, form: values, revision: at });
      held.adopt(stored.revision);
      setSaved(true);
      toast.success(t(service ? "services.saved" : "services.created"));
      onSaved();
    } catch (error) {
      if (error instanceof ValidationError) {
        const { placed, unplaced } = byField(error.fields, Object.keys(emptyServiceForm(environments)), form.getValues("rows"));
        for (const { path, message } of placed) {
          form.setError(path as Path<ServiceFormValues>, { message });
        }
        setUnplaced(unplaced);
      } else if (error instanceof ConflictError) {
        setConflict(true);
        onConflict();
      } else {
        toast.error(t("errors.generic"));
      }
    }
  });
  const submit = send(held.revision);
  const reload = () => {
    held.catchUp();
    form.reset(initialOf(service, environments));
    setConflict(false);
  };

  const errors = form.formState.errors;
  return (
    <form onSubmit={submit} className="grid gap-6" noValidate>
      <fieldset disabled={!editable} className="contents">
        {conflict ? <ConflictNotice pending={held.latest === held.revision || save.isPending} onReload={reload} onOverwrite={() => void send(held.catchUp())()} /> : null}
        {unplaced.length > 0 ? (
          <ErrorNotice title={t("serviceForm.notSaved")} description={unplaced.map((error) => `${error.field}: ${error.message}`).join("\n")} />
        ) : null}
        <div className="grid items-start gap-6 lg:grid-cols-2">
          <SectionCard title={t("serviceForm.identity")}>
            <div className="grid gap-4">
              <FormField id="service-name" label={t("serviceForm.name")} error={errors.name?.message}>
                <Input
                  id="service-name"
                  autoFocus
                  {...form.register("name", {
                    onChange: (event: { target: { value: string } }) => {
                      if (idFollowsName) {
                        form.setValue("id", uniqueId(slugOf(event.target.value), taken), { shouldDirty: true });
                      }
                    },
                  })}
                />
              </FormField>
              <FormField id="service-id" label={t("serviceForm.id")} hint={t("serviceForm.idHint")} error={errors.id?.message}>
                <Input
                  id="service-id"
                  autoComplete="off"
                  spellCheck={false}
                  {...form.register("id", {
                    onChange: (event: { target: { value: string } }) => setIdFollowsName(service === null && event.target.value.trim() === ""),
                  })}
                />
              </FormField>
              <FormField id="service-group" label={t("serviceForm.group")} hint={t("serviceForm.groupHint")} optional error={errors.group?.message}>
                <Controller
                  control={form.control}
                  name="group"
                  render={({ field }) => (
                    <TagInput
                      id="service-group"
                      values={field.value.trim() === "" ? [] : [field.value]}
                      onChange={(values) => field.onChange(values[0] ?? "")}
                      onBlur={field.onBlur}
                      suggestions={groups}
                      max={1}
                      removeLabel={(value) => t("tagInput.remove", { value })}
                      createLabel={(value) => t("tagInput.create", { value })}
                    />
                  )}
                />
              </FormField>
              <IconField form={form} />
              <FormField id="service-description" label={t("serviceForm.serviceDescription")} optional error={errors.description?.message}>
                <Input id="service-description" {...form.register("description")} />
              </FormField>
            </div>
          </SectionCard>
          <SectionCard title={t("serviceForm.probe")}>
            <ProbeFields form={form} />
          </SectionCard>
        </div>
        <SectionCard title={t("serviceForm.rows.title")} description={t("serviceForm.rows.description")}>
          <AddressRows form={form} />
        </SectionCard>
        <SectionCard title={t("serviceForm.details")}>
          <DetailsFields form={form} />
        </SectionCard>
        <div className="glass-panel sticky bottom-3 z-20 flex justify-end gap-2 rounded-xl px-4 py-3">
          <Button asChild variant="outline">
            <Link href={cancelHref}>{t("common.cancel")}</Link>
          </Button>
          {editable ? (
            <Button type="submit" disabled={form.formState.isSubmitting}>
              {form.formState.isSubmitting ? t("common.saving") : t("common.save")}
            </Button>
          ) : null}
        </div>
      </fieldset>
    </form>
  );
}
