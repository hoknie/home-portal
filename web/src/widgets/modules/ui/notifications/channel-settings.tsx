"use client";

import { useTranslations } from "next-intl";
import { useState } from "react";
import { toast } from "sonner";

import { type NotificationChannel, useChangeChannel } from "@/entities/notification";
import { useSecretNames } from "@/entities/workflow";
import { ConflictError, ValidationError } from "@/shared/api";
import { ErrorNotice } from "@/shared/ui/error-notice";
import { FormField } from "@/shared/ui/form-field";
import { Button, Input, Label, Switch } from "@/shared/ui/primitives";

export const SECRET_SETTING = "secret";

const SELECT =
  "h-9 w-full rounded-md border border-input bg-glass-tint px-3 text-sm shadow-xs outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50";

export function ChannelSettings({ channel, revision }: { channel: NotificationChannel; revision: string | null }) {
  const t = useTranslations();
  const change = useChangeChannel();
  const secrets = useSecretNames();
  const [draft, setDraft] = useState<Record<string, unknown>>(channel.settings);
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [conflict, setConflict] = useState(false);
  const label = (key: string) => (t.has(`notifications.settings.${key}` as "notifications.settings.secret") ? t(`notifications.settings.${key}` as "notifications.settings.secret") : key);
  const set = (key: string, value: unknown) => setDraft((current) => ({ ...current, [key]: value }));
  const save = async () => {
    setErrors({});
    setConflict(false);
    try {
      const saved = await change.mutateAsync({ name: channel.name, settings: draft, revision });
      setDraft(saved.data.channels.find((candidate) => candidate.name === channel.name)?.settings ?? draft);
      toast.success(t("notifications.saved"));
    } catch (error) {
      if (error instanceof ValidationError) {
        setErrors(Object.fromEntries(error.fields.map((field) => [field.field.split(".").pop() ?? field.field, field.message])));
      } else if (error instanceof ConflictError) {
        setConflict(true);
      } else {
        toast.error(t("errors.generic"));
      }
    }
  };
  const secretNames = [...new Set([...(secrets.data ?? []).map((secret) => secret.name), ...(typeof draft[SECRET_SETTING] === "string" ? [draft[SECRET_SETTING]] : [])])];
  return (
    <form
      className="grid gap-4 border-t pt-4"
      noValidate
      aria-label={t("notifications.settings.title")}
      onSubmit={(event) => {
        event.preventDefault();
        void save();
      }}
    >
      {conflict ? <ErrorNotice title={t("errors.conflict")} /> : null}
      {Object.entries(draft).map(([key, value]) => {
        const id = `channel-${channel.name}-${key}`;
        if (typeof value === "boolean") {
          return (
            <div key={key} className="flex items-center justify-between gap-4">
              <Label htmlFor={id}>{label(key)}</Label>
              <Switch id={id} checked={value} onCheckedChange={(next) => set(key, next)} />
            </div>
          );
        }
        return (
          <FormField key={key} id={id} label={label(key)} error={errors[key]}>
            {key === SECRET_SETTING ? (
              <select id={id} className={SELECT} value={typeof value === "string" ? value : ""} onChange={(event) => set(key, event.target.value)}>
                <option value="">{t("notifications.settings.noSecret")}</option>
                {secretNames.map((name) => (
                  <option key={name} value={name}>
                    {name}
                  </option>
                ))}
              </select>
            ) : (
              <Input id={id} spellCheck={false} autoComplete="off" value={value === null || value === undefined ? "" : String(value)} onChange={(event) => set(key, event.target.value)} />
            )}
          </FormField>
        );
      })}
      <div>
        <Button type="submit" disabled={change.isPending}>
          {t("notifications.settings.save")}
        </Button>
      </div>
    </form>
  );
}
