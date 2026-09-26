"use client";

import { Pencil, SearchX } from "lucide-react";
import Link from "next/link";
import { useSearchParams } from "next/navigation";
import { useTranslations } from "next-intl";

import { TokenActions } from "@/features/automation-builder";
import { StopRunButton } from "@/features/stop-run";
import { RunTable, useAutomations, useRuns } from "@/entities/automation";
import { absoluteAddress, useWebhooks } from "@/entities/webhook";
import { routes } from "@/shared/config";
import { EmptyState } from "@/shared/ui/empty-state";
import { ErrorNotice } from "@/shared/ui/error-notice";
import { KvList, KvRow } from "@/shared/ui/kv-list";
import { PageHeader } from "@/shared/ui/page-header";
import { Badge, Button, Skeleton } from "@/shared/ui/primitives";
import { RelativeTime } from "@/shared/ui/relative-time";
import { SectionCard } from "@/shared/ui/section-card";
import { CopyLine } from "@/shared/ui/copy-line";
import { TagList } from "@/shared/ui/tag-list";

export const ID_PARAMETER = "id";
export const LAST_RUNS = 10;

export function WebhookDetailsScreen() {
  const t = useTranslations();
  const id = useSearchParams().get(ID_PARAMETER) ?? "";
  const webhooks = useWebhooks();
  const automations = useAutomations();
  const runs = useRuns({ webhook: id }, true, id !== "");
  if (!webhooks.data) {
    return webhooks.error ? (
      <ErrorNotice title={t("errors.loadFailed")} description={webhooks.error.message} onRetry={() => void webhooks.refetch()} />
    ) : (
      <Skeleton className="h-96 w-full" aria-busy="true" />
    );
  }
  const webhook = webhooks.data.data.webhooks.find((candidate) => candidate.id === id);
  if (!webhook) {
    return (
      <EmptyState
        icon={SearchX}
        title={t("webhooks.notFound")}
        description={t("webhooks.notFoundHint")}
        action={
          <Button asChild variant="outline">
            <Link href={routes.adminWebhooks}>{t("webhooks.backToWebhooks")}</Link>
          </Button>
        }
      />
    );
  }
  const url = typeof window === "undefined" ? webhook.address : absoluteAddress(webhook.address, window.location.origin);
  const listeners = (automations.data?.data.automations ?? []).filter(
    (automation) => automation.when.event === "webhook.received" && (automation.when.webhooks.length === 0 || automation.when.webhooks.includes(webhook.id)),
  );
  const titleOf = (runId: string) => automations.data?.data.automations.find((automation) => automation.id === runId)?.title ?? (runId === webhook.id ? webhook.title : runId);
  const edit = (
    <Button asChild variant="outline">
      <Link href={routes.editWebhook(webhook.id)}>
        <Pencil aria-hidden />
        {t("common.edit")}
      </Link>
    </Button>
  );
  return (
    <div className="grid gap-8">
      <PageHeader title={webhook.title} description={t("webhooks.detailsDescription")} actions={edit} />
      <div className="grid items-start gap-6 lg:grid-cols-2">
        <SectionCard title={t("webhooks.identity")}>
          <KvList>
            <KvRow label={t("webhooks.address")}>
              <CopyLine text={url} />
            </KvRow>
            <KvRow label={t("tags.label")}>
              {webhook.tags.length > 0 ? <TagList tags={webhook.tags} /> : <span className="text-muted-foreground">{t("tags.none")}</span>}
            </KvRow>
            <KvRow label={t("webhooks.enabledLabel")}>{t(webhook.enabled ? "automations.enabled" : "automations.disabled")}</KvRow>
            <KvRow label={t("webhooks.variables")}>
              {webhook.variables.length === 0 ? (
                <span className="text-muted-foreground">{t("webhooks.noVariables")}</span>
              ) : (
                <span className="flex flex-wrap gap-1">
                  {webhook.variables.map((variable) => (
                    <Badge key={variable} variant="outline" className="font-mono">
                      {variable}
                    </Badge>
                  ))}
                </span>
              )}
            </KvRow>
            <KvRow label={t("webhooks.columns.lastReceived")}>
              {webhook.last_received ? (
                <span className="flex gap-2">
                  <RelativeTime moment={webhook.last_received.at} />
                  <span className="text-muted-foreground">{webhook.last_received.status}</span>
                </span>
              ) : (
                t("webhooks.neverReceived")
              )}
            </KvRow>
          </KvList>
          <div className="mt-4">
            <TokenActions webhook={webhook} revision={webhooks.data.revision} />
          </div>
        </SectionCard>
        <SectionCard title={t("webhooks.action")}>
          <div className="grid gap-3 text-sm">
            <p className="font-medium">{t(`webhooks.actions.${webhook.action}`)}</p>
            {webhook.run ? (
              <pre className="overflow-x-auto rounded-md border border-glass-edge bg-glass-tint p-3 font-mono text-xs whitespace-pre-wrap break-all">
                {[webhook.run.script, ...webhook.run.args].join(" ")}
              </pre>
            ) : listeners.length === 0 ? (
              <p className="text-muted-foreground">{t("webhooks.noListeners")}</p>
            ) : (
              <ul className="grid gap-1">
                {listeners.map((automation) => (
                  <li key={automation.id}>
                    <Link href={routes.editAutomation(automation.id)} className="hover:underline">
                      {automation.title}
                    </Link>
                  </li>
                ))}
              </ul>
            )}
          </div>
        </SectionCard>
      </div>
      <SectionCard title={t("webhooks.lastRuns", { count: LAST_RUNS })} flush>
        {runs.data ? <RunTable runs={runs.data.runs.slice(0, LAST_RUNS)} titleOf={titleOf} actionsOf={(run) => <StopRunButton run={run} title={titleOf(run.automation)} />} /> : <Skeleton className="m-4 h-32" aria-busy="true" />}
      </SectionCard>
    </div>
  );
}
