"use client";

import { useTranslations } from "next-intl";
import { useState } from "react";
import { toast } from "sonner";

import { Allowed, useCan } from "@/entities/session";
import { type NotificationChannel, useChangeChannel } from "@/entities/notification";
import { useSecretNames } from "@/entities/workflow";
import { ConflictError, ValidationError } from "@/shared/api";
import { useEditorRevision } from "@/shared/lib/editor-revision";
import { ConflictNotice } from "@/shared/ui/conflict-notice";
import { Button, FormField, Input, Label, NativeSelect, Switch } from "@/shared/ui/kit";

export const SECRET_SETTING = "secret";


export function ChannelSettings({ channel, revision }: { channel: NotificationChannel; revision: string | null }) {
  const t = useTranslations();
  const can = useCan();
  const change = useChangeChannel();
  const secrets = useSecretNames();
  const [draft, setDraft] = useState<Record<string, unknown>>(channel.settings);
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [conflict, setConflict] = useState(false);
  const held = useEditorRevision(revision);
  const label = (key: string) => (t.has(`notifications.settings.${key}` as "notifications.settings.secret") ? t(`notifications.settings.${key}` as "notifications.settings.secret") : key);
  const set = (key: string, value: unknown) => setDraft((current) => ({ ...current, [key]: value }));
  const save = async (at: string | null = held.revision) => {
    setErrors({});
    setConflict(false);
    try {
      const saved = await change.mutateAsync({ name: channel.name, settings: draft, revision: at });
      held.adopt(saved.revision);
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
      {conflict ? (
        <ConflictNotice
          pending={held.latest === held.revision || change.isPending}
          onReload={() => {
            held.catchUp();
            setDraft(channel.settings);
            setConflict(false);
          }}
          onOverwrite={() => void save(held.catchUp())}
        />
      ) : null}
      <fieldset disabled={!can("notifications", "update")} className="contents">
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
                <NativeSelect id={id} value={typeof value === "string" ? value : ""} onChange={(event) => set(key, event.target.value)}>
                  <option value="">{t("notifications.settings.noSecret")}</option>
                  {secretNames.map((name) => (
                    <option key={name} value={name}>
                      {name}
                    </option>
                  ))}
                </NativeSelect>
              ) : (
                <Input id={id} spellCheck={false} autoComplete="off" value={value === null || value === undefined ? "" : String(value)} onChange={(event) => set(key, event.target.value)} />
              )}
            </FormField>
          );
        })}
      </fieldset>
      <div>
        <Allowed area="notifications" action="update">
          <Button type="submit" disabled={change.isPending}>
            {t("notifications.settings.save")}
          </Button>
        </Allowed>
      </div>
    </form>
  );
}
