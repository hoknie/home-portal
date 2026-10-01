"use client";

import { Info, Lock, Plus, X } from "lucide-react";
import { useTranslations } from "next-intl";
import { Controller, type UseFormReturn, useFieldArray } from "react-hook-form";

import { useProxy } from "@/entities/proxy";
import { type ServiceForm, row } from "@/entities/service";
import { MultiSelect } from "@/shared/ui/multi-select";
import { Button, Input } from "@/shared/ui/primitives";

import { ProxySettings } from "./proxy-settings";

const SELECT =
  "h-9 w-full rounded-md border border-input bg-glass-tint px-3 text-sm shadow-xs outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:opacity-50";
const COLUMNS = "md:grid-cols-[minmax(0,2fr)_minmax(0,1.2fr)_minmax(0,2.4fr)_max-content_2.25rem]";
const SPAN = "md:col-span-full md:grid-cols-subgrid";

type Props = { form: UseFormReturn<ServiceForm> };

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

export function AddressRows({ form }: Props) {
  const t = useTranslations("serviceForm.rows");
  const proxy = useProxy();
  const rows = useFieldArray({ control: form.control, name: "rows" });
  const values = form.watch("rows");
  const choices = form.watch("kept.choices");
  const editable = values.filter((current) => !current.locked).length;
  const proxied = values.some((current) => current.proxied);
  const errors = form.formState.errors.rows;
  const takenElsewhere = (index: number) => new Set(values.flatMap((current, position) => (position === index || current.locked ? [] : current.environments)));
  return (
    <div className="grid gap-4">
      {proxy.data && !proxy.data.data.enabled ? (
        <p role="note" className="flex items-start gap-2 text-sm text-muted-foreground">
          <Info className="mt-0.5 size-4 shrink-0" aria-hidden />
          {t("proxyOff")}
        </p>
      ) : null}
      <div className={`grid gap-3 ${COLUMNS}`} data-address-table="">
        <div className={`hidden text-xs font-medium text-muted-foreground md:grid ${SPAN}`} aria-hidden>
          <span>{t("environments")}</span>
          <span>{t("signIn")}</span>
          <span>{t("address")}</span>
          <span>{t("throughProxy")}</span>
          <span />
        </div>
        <ol className={`grid gap-3 ${SPAN}`} aria-label={t("title")}>
          {rows.fields.map((field, index) => {
            const current = values[index] ?? row();
            const rowErrors = errors?.[index];
            const taken = takenElsewhere(index);
            const label = index === 0 ? t("main") : t("number", { number: index + 1 });
            return (
              <li key={field.id} aria-label={label} className={`grid items-start gap-3 rounded-lg border border-glass-edge bg-glass-tint p-3 md:border-0 md:bg-transparent md:p-0 ${SPAN}`}>
                <div className="grid gap-1">
                  <Controller
                    control={form.control}
                    name={`rows.${index}.environments`}
                    render={({ field: environments }) => (
                      <MultiSelect
                        id={`row-${index}-environments`}
                        label={t("environmentsOf", { row: label })}
                        placeholder={index === 0 ? t("onlyProbed") : t("noEnvironments")}
                        disabled={current.locked}
                        invalid={Boolean(rowErrors?.environments)}
                        options={choices.map((choice) => ({ value: choice, label: choice, disabled: !environments.value.includes(choice) && taken.has(choice) }))}
                        values={environments.value}
                        onChange={environments.onChange}
                      />
                    )}
                  />
                  <Message text={rowErrors?.environments?.message} />
                </div>
                <div className="grid gap-1">
                  <label className="sr-only" htmlFor={`row-${index}-sign-in`}>
                    {t("signInOf", { row: label })}
                  </label>
                  <Controller
                    control={form.control}
                    name={`rows.${index}.sign_in`}
                    render={({ field: signIn }) => (
                      <select
                        id={`row-${index}-sign-in`}
                        className={SELECT}
                        disabled={current.locked}
                        value={signIn.value ? "portal" : "none"}
                        onChange={(event) => signIn.onChange(event.target.value === "portal")}
                      >
                        <option value="none">{t("signInNone")}</option>
                        <option value="portal">{t("signInPortal")}</option>
                      </select>
                    )}
                  />
                  <Message text={rowErrors?.sign_in?.message} />
                </div>
                <div className="grid gap-1">
                  <label className="sr-only" htmlFor={`row-${index}-address`}>
                    {t("addressOf", { row: label })}
                  </label>
                  <Input
                    id={`row-${index}-address`}
                    type="url"
                    inputMode="url"
                    spellCheck={false}
                    readOnly={current.locked}
                    placeholder={index === 0 ? "http://192.168.1.50" : current.proxied ? "https://media.example.com" : ""}
                    aria-invalid={rowErrors?.address ? true : undefined}
                    {...form.register(`rows.${index}.address`)}
                  />
                  {index === 0 ? <p className="text-xs text-muted-foreground">{t("mainHint")}</p> : null}
                  {current.locked ? (
                    <p className="flex items-center gap-1 text-xs text-muted-foreground">
                      <Lock className="size-3" aria-hidden />
                      {t("locked")}
                    </p>
                  ) : null}
                  <Message text={rowErrors?.address?.message} />
                </div>
                <div className="grid gap-1">
                  <label className="flex min-h-9 items-center gap-2 text-sm">
                    <Controller
                      control={form.control}
                      name={`rows.${index}.proxied`}
                      render={({ field: through }) => (
                        <input
                          type="checkbox"
                          className="size-4 accent-primary"
                          aria-describedby={index === 0 ? "row-main-proxy-hint" : undefined}
                          disabled={index === 0 || current.locked || (!through.value && proxied)}
                          checked={through.value}
                          onChange={(event) => through.onChange(event.target.checked)}
                        />
                      )}
                    />
                    <span className="md:sr-only">{t("throughProxyOf", { row: label })}</span>
                  </label>
                  {index === 0 ? (
                    <span id="row-main-proxy-hint" className="sr-only">
                      {t("mainNotProxied")}
                    </span>
                  ) : null}
                  <Message text={rowErrors?.proxied?.message} />
                </div>
                {index === 0 || current.locked ? (
                  <span />
                ) : (
                  <Button type="button" variant="ghost" size="icon" aria-label={t("remove", { row: label })} onClick={() => rows.remove(index)}>
                    <X aria-hidden />
                  </Button>
                )}
              </li>
            );
          })}
        </ol>
      </div>
      <Button
        type="button"
        variant="outline"
        size="sm"
        className="justify-self-start"
        disabled={editable >= choices.length}
        onClick={() => rows.append(row())}
      >
        <Plus aria-hidden />
        {t("add")}
      </Button>
      {proxied ? <ProxySettings form={form} /> : null}
    </div>
  );
}
