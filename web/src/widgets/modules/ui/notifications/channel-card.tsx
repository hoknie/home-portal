"use client";

import { useTranslations } from "next-intl";
import { useState } from "react";

import { Allowed } from "@/entities/session";
import { type NotificationChannel, useSendTest } from "@/entities/notification";
import { ConflictError } from "@/shared/api";
import { Badge, Button, KvList, KvRow } from "@/shared/ui/kit";
import { RelativeTime } from "@/shared/ui/relative-time";
import { SectionCard } from "@/shared/ui/section-card";

import { ChannelSettings } from "./channel-settings";

const BADGE = { ready: "default", disabled: "secondary", missing: "destructive" } as const;

export type ChannelCardProps = { channel: NotificationChannel; revision: string | null; moduleOn: boolean };

export function ChannelCard({ channel, revision, moduleOn }: ChannelCardProps) {
  const t = useTranslations("notifications");
  const test = useSendTest();
  const [outcome, setOutcome] = useState<string | null>(null);
  const ready = moduleOn && channel.readiness === "ready";
  const send = async () => {
    setOutcome(null);
    try {
      const delivery = await test.mutateAsync(channel.name);
      setOutcome(delivery.delivered ? t("test.sent") : t("test.failed", { error: delivery.error ?? "" }));
    } catch (error) {
      setOutcome(error instanceof ConflictError ? t("test.unavailable") : t("test.failed", { error: error instanceof Error ? error.message : "" }));
    }
  };
  return (
    <SectionCard
      title={t.has(`channels.${channel.name}` as "channels.telegram") ? t(`channels.${channel.name}` as "channels.telegram") : channel.name}
      badge={<Badge variant={BADGE[channel.readiness]}>{t(`readiness.${channel.readiness}`)}</Badge>}
      actions={
        <Allowed area="notifications" action="update">
          <Button type="button" variant="outline" size="sm" onClick={() => void send()} disabled={!ready || test.isPending} title={ready ? undefined : t("test.unavailable")}>
            {t("test.send")}
          </Button>
        </Allowed>
      }
    >
      <div className="grid gap-5">
        {outcome ? (
          <p role="status" aria-live="polite" className="text-sm">
            {outcome}
          </p>
        ) : null}
        {channel.missing ? (
          <p className="rounded-lg border border-destructive/40 bg-destructive/10 p-3 text-sm">
            {t("missing", { field: channel.missing.field, message: channel.missing.message })}
          </p>
        ) : null}
        <KvList>
          <KvRow label={t("lastDelivery")}>
            {channel.last_delivery ? <RelativeTime moment={channel.last_delivery.at} /> : t("never")}
          </KvRow>
          {channel.last_error ? (
            <KvRow label={t("lastError")}>
              <span className="grid gap-0.5">
                <span className="text-destructive">{channel.last_error.message}</span>
                <span className="text-xs font-normal text-muted-foreground">
                  <RelativeTime moment={channel.last_error.at} />
                </span>
              </span>
            </KvRow>
          ) : null}
          <KvRow label={t("queue")}>{t("queueCounts", { queued: channel.queued, dropped: channel.dropped })}</KvRow>
        </KvList>
        <ChannelSettings channel={channel} revision={revision} />
      </div>
    </SectionCard>
  );
}
