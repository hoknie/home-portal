"use client";

import { useTranslations } from "next-intl";

import { useNotifications } from "@/entities/notification";
import { ErrorNotice } from "@/shared/ui/error-notice";
import { ModuleOffNotice } from "@/shared/ui/module-off-notice";
import { useTrail } from "@/shared/lib/breadcrumbs";
import { PageHeader } from "@/shared/ui/page-header";
import { Skeleton } from "@/shared/ui/primitives";

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
        <>
          <div className="grid gap-4 lg:grid-cols-2">
            {data.channels.map((channel) => (
              <ChannelCard key={channel.name} channel={channel} revision={revision} moduleOn={data.enabled} />
            ))}
          </div>
          <RulesForm rules={data.rules} revision={revision} />
        </>
      ) : notifications.error ? null : (
        <Skeleton className="h-64 w-full" aria-busy="true" />
      )}
    </div>
  );
}
