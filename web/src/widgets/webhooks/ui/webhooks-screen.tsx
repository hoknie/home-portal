"use client";

import { Pencil, Plus, Webhook as WebhookIcon } from "lucide-react";
import Link from "next/link";
import { useTranslations } from "next-intl";
import { useState } from "react";

import { DeleteWebhookButton } from "@/features/delete-webhook";
import { type Webhook, absoluteAddress, shortAddress, useWebhooks } from "@/entities/webhook";
import { routes } from "@/shared/config";
import { type Column, DataTable } from "@/shared/ui/data-table";
import { EmptyState } from "@/shared/ui/empty-state";
import { ErrorNotice } from "@/shared/ui/error-notice";
import { PageHeader } from "@/shared/ui/page-header";
import { Badge, Button, Skeleton, Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/shared/ui/primitives";
import { RelativeTime } from "@/shared/ui/relative-time";
import { SectionCard } from "@/shared/ui/section-card";
import { TagFilter, distinctTags, stillChosen, tagsMatch } from "@/shared/ui/tag-filter";
import { TagList } from "@/shared/ui/tag-list";

function useColumns(revision: string | null): Column<Webhook>[] {
  const t = useTranslations();
  const origin = typeof window === "undefined" ? "" : window.location.origin;
  return [
    {
      key: "title",
      header: t("webhooks.columns.title"),
      cell: (webhook) => (
        <div className="grid min-w-0 gap-1">
          <Link href={routes.webhookDetails(webhook.id)} className="truncate font-medium hover:underline">
            {webhook.title}
          </Link>
          <TooltipProvider>
            <Tooltip>
              <TooltipTrigger asChild>
                <code tabIndex={0} className="w-fit font-mono text-xs text-muted-foreground">
                  {shortAddress(webhook.address)}
                </code>
              </TooltipTrigger>
              <TooltipContent className="font-mono">{absoluteAddress(webhook.address, origin)}</TooltipContent>
            </Tooltip>
          </TooltipProvider>
          <TagList tags={webhook.tags} />
        </div>
      ),
    },
    {
      key: "access",
      header: t("webhooks.columns.access"),
      cell: (webhook) => <Badge variant={webhook.protected ? "outline" : "destructive"}>{t(webhook.protected ? "webhooks.protected" : "webhooks.open")}</Badge>,
    },
    {
      key: "action",
      header: t("webhooks.columns.action"),
      hideBelow: "md",
      cell: (webhook) => (
        <span className="grid gap-0.5">
          <span>{t(`webhooks.actions.${webhook.action}`)}</span>
          {webhook.run ? <span className="font-mono text-xs text-muted-foreground">{webhook.run.script}</span> : null}
        </span>
      ),
    },
    {
      key: "last",
      header: t("webhooks.columns.lastReceived"),
      hideBelow: "sm",
      cell: (webhook) =>
        webhook.last_received ? (
          <span className="flex gap-2 text-xs text-muted-foreground">
            <RelativeTime moment={webhook.last_received.at} />
            <span>{webhook.last_received.status}</span>
          </span>
        ) : (
          <span className="text-xs text-muted-foreground">{t("webhooks.neverReceived")}</span>
        ),
    },
    {
      key: "actions",
      header: t("webhooks.columns.actions"),
      align: "end",
      cell: (webhook) => (
        <div className="flex justify-end gap-1">
          <Button asChild variant="ghost" size="icon">
            <Link href={routes.editWebhook(webhook.id)} aria-label={t("common.edit")}>
              <Pencil aria-hidden />
            </Link>
          </Button>
          <DeleteWebhookButton webhook={webhook} revision={revision} />
        </div>
      ),
    },
  ];
}

export function WebhooksScreen() {
  const t = useTranslations();
  const webhooks = useWebhooks();
  const columns = useColumns(webhooks.data?.revision ?? null);
  const [picked, setChosen] = useState<string[]>([]);
  const list = webhooks.data?.data.webhooks ?? [];
  const tags = distinctTags(list.map((webhook) => webhook.tags));
  const chosen = stillChosen(picked, tags);
  const add = (
    <Button asChild>
      <Link href={routes.newWebhook}>
        <Plus aria-hidden />
        {t("webhooks.add")}
      </Link>
    </Button>
  );
  return (
    <div className="grid gap-8">
      <PageHeader title={t("webhooks.title")} description={t("webhooks.subtitle")} actions={add} />
      {webhooks.error && !webhooks.data ? (
        <ErrorNotice title={t("errors.loadFailed")} description={webhooks.error.message} onRetry={() => void webhooks.refetch()} />
      ) : null}
      {webhooks.data ? (
        <SectionCard flush actions={<TagFilter label={t("tags.filter")} tags={tags} selected={chosen} onChange={setChosen} />}>
          <DataTable
            columns={columns}
            rows={list.filter((webhook) => tagsMatch(webhook.tags, chosen))}
            rowKey={(webhook) => webhook.id}
            empty={
              chosen.length > 0 ? (
                <EmptyState icon={WebhookIcon} title={t("tags.noMatches")} description={t("tags.noMatchesHint")} />
              ) : (
                <EmptyState icon={WebhookIcon} title={t("webhooks.empty")} description={t("webhooks.emptyHint")} action={add} />
              )
            }
          />
        </SectionCard>
      ) : webhooks.error ? null : (
        <Skeleton className="h-64 w-full" aria-busy="true" />
      )}
    </div>
  );
}
