"use client";

import { Info } from "lucide-react";
import { useTranslations } from "next-intl";
import { Controller, type UseFormReturn } from "react-hook-form";

import { useEnvironment } from "@/entities/environment";
import { useProxy } from "@/entities/proxy";
import { PUBLICATION_TLS, type ServiceForm } from "@/entities/service";
import { FormField } from "@/shared/ui/form-field";
import { Input, Label, Switch } from "@/shared/ui/primitives";

import { FieldGroup } from "./field-group";

const SELECT =
  "h-9 w-full rounded-md border border-input bg-glass-tint px-3 text-sm shadow-xs outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50";
const INTERNET = "internet";

type ChoicesProps = {
  form: UseFormReturn<ServiceForm>;
  name: "publication.environments" | "publication.auth";
  legend: string;
  choices: string[];
  error?: string;
};

function Choices({ form, name, legend, choices, error }: ChoicesProps) {
  const t = useTranslations();
  const id = name.replace(".", "-");
  const message = error && t.has(error as Parameters<typeof t.has>[0]) ? t(error as Parameters<typeof t>[0]) : error;
  return (
    <Controller
      control={form.control}
      name={name}
      render={({ field }) => (
        <div role="group" aria-labelledby={`${id}-label`} className="grid content-start gap-2">
          <Label asChild>
            <span id={`${id}-label`}>{legend}</span>
          </Label>
          <div className="flex flex-wrap gap-x-4">
            {choices.map((choice) => (
              <label key={choice} className="flex min-h-9 items-center gap-2 text-sm">
                <input
                  type="checkbox"
                  className="size-4 accent-primary"
                  checked={field.value.includes(choice)}
                  onChange={(event) =>
                    field.onChange(event.target.checked ? [...field.value, choice] : field.value.filter((value) => value !== choice))
                  }
                />
                {choice}
              </label>
            ))}
          </div>
          {message ? (
            <p role="alert" className="text-sm text-destructive">
              {message}
            </p>
          ) : null}
        </div>
      )}
    />
  );
}

export function PublicationFields({ form }: { form: UseFormReturn<ServiceForm> }) {
  const t = useTranslations();
  const environment = useEnvironment();
  const proxy = useProxy();
  const errors = form.formState.errors.publication;
  const configured = environment.data?.environments ?? [];
  const choices = configured.includes(INTERNET) ? configured : [...configured, INTERNET];
  const tls = form.watch("publication.tls");
  const upstream = form.watch("publication.upstream");
  const url = form.watch("url");
  const https = (upstream.trim() === "" ? url : upstream).trim().startsWith("https://");
  return (
    <div className="grid gap-6">
      {proxy.data && !proxy.data.data.enabled ? (
        <p role="note" className="flex items-start gap-2 text-sm text-muted-foreground">
          <Info className="mt-0.5 size-4 shrink-0" aria-hidden />
          {t("serviceForm.publicationDisabled")}
        </p>
      ) : null}
      <FieldGroup id="publication-address" title={t("serviceForm.publicationGroups.address")}>
        <div className="grid gap-4 sm:grid-cols-2">
          <FormField id="publication-host" label={t("serviceForm.publicationHost")} hint={t("serviceForm.publicationHostHint")} optional error={errors?.host?.message}>
            <Input id="publication-host" spellCheck={false} autoComplete="off" {...form.register("publication.host")} />
          </FormField>
          <FormField
            id="publication-upstream"
            label={t("serviceForm.publicationUpstream")}
            hint={t("serviceForm.publicationUpstreamHint")}
            optional
            error={errors?.upstream?.message}
          >
            <Input id="publication-upstream" type="url" inputMode="url" spellCheck={false} {...form.register("publication.upstream")} />
          </FormField>
        </div>
      </FieldGroup>
      <FieldGroup id="publication-access" title={t("serviceForm.publicationGroups.access")}>
        <div className="grid gap-4 sm:grid-cols-2">
          <Choices
            form={form}
            name="publication.environments"
            legend={t("serviceForm.publicationEnvironments")}
            choices={choices}
            error={errors?.environments?.message}
          />
          <Choices form={form} name="publication.auth" legend={t("serviceForm.publicationAuth")} choices={choices} error={errors?.auth?.message} />
        </div>
      </FieldGroup>
      <FieldGroup id="publication-certificate" title={t("serviceForm.publicationGroups.certificate")}>
        <div className="grid gap-4 sm:grid-cols-2">
          <FormField id="publication-tls" label={t("serviceForm.publicationTls")} hint={t(`serviceForm.publicationTlsHints.${tls}`)}>
            <select id="publication-tls" className={SELECT} {...form.register("publication.tls")}>
              {PUBLICATION_TLS.map((mode) => (
                <option key={mode} value={mode}>
                  {t(`serviceForm.publicationTlsModes.${mode}`)}
                </option>
              ))}
            </select>
          </FormField>
          {tls === "acme" ? (
            <FormField id="publication-email" label={t("serviceForm.publicationEmail")} optional error={errors?.email?.message}>
              <Input id="publication-email" type="email" {...form.register("publication.email")} />
            </FormField>
          ) : null}
        </div>
        {tls === "files" ? (
          <div className="grid gap-4 sm:grid-cols-2">
            <FormField id="publication-certificate" label={t("serviceForm.publicationCertificate")} error={errors?.certificate?.message}>
              <Input id="publication-certificate" spellCheck={false} {...form.register("publication.certificate")} />
            </FormField>
            <FormField id="publication-key" label={t("serviceForm.publicationKey")} error={errors?.key?.message}>
              <Input id="publication-key" spellCheck={false} {...form.register("publication.key")} />
            </FormField>
          </div>
        ) : null}
        {https ? (
          <div className="flex items-center justify-between gap-4">
            <Label htmlFor="publication-verify">{t("serviceForm.publicationVerify")}</Label>
            <Controller
              control={form.control}
              name="publication.upstream_verify"
              render={({ field }) => <Switch id="publication-verify" checked={field.value} onCheckedChange={field.onChange} />}
            />
          </div>
        ) : null}
      </FieldGroup>
    </div>
  );
}
