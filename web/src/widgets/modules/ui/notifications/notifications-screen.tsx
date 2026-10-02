"use client";

import { useTranslations } from "next-intl";

import { useNotifications } from "@/entities/notification";
import { Appear, ErrorNotice, SkeletonCard, SkeletonForm } from "@/shared/ui/kit";
import { ModuleOffNotice } from "@/shared/ui/module-off-notice";
import { useTrail } from "@/shared/lib/breadcrumbs";
import { PageHeader } from "@/shared/ui/page-header";

import { ChannelCard } from "./channel-card";
import { RulesForm } from "./rules-form";

export function NotificationsScreen() {
  const trail = useTrail();
  const t = useTranslations();
  const notifications = useNotifications({ live: true });
  const data = notifications.data?.data;
  const revision = notifications.data?.revision ?? null;
  return (
    <div className="grid gap-8">
      <PageHeader breadcrumbs={trail.of(trail.section("notifications"))} title={t("notifications.title")} description={t("notifications.subtitle")} />
      {data && !data.enabled ? <ModuleOffNotice name={t("modules.names.notifications")} /> : null}
      {notifications.error && !data ? (
        <ErrorNotice title={t("errors.loadFailed")} description={notifications.error.message} onRetry={() => void notifications.refetch()} />
      ) : null}
      {data ? (
        <Appear className="grid gap-8">
          <div className="grid gap-4 lg:grid-cols-2">
            {data.channels.map((channel) => (
              <ChannelCard key={channel.name} channel={channel} revision={revision} moduleOn={data.enabled} />
            ))}
          </div>
          <RulesForm rules={data.rules} revision={revision} />
        </Appear>
      ) : notifications.error ? null : (
        <div className="grid gap-8" data-skeleton="notifications" aria-busy="true">
          <div className="grid gap-4 lg:grid-cols-2">
            <SkeletonForm fields={2} />
            <SkeletonForm fields={2} />
          </div>
          <SkeletonCard lines={3} />
        </div>
      )}
    </div>
  );
}
