"use client";

import { SearchX } from "lucide-react";
import Link from "next/link";
import { useRouter, useSearchParams } from "next/navigation";
import { useTranslations } from "next-intl";

import { WebhookForm } from "@/features/automation-builder";
import { useCatalogue, useScripts } from "@/entities/automation";
import { useWebhooks } from "@/entities/webhook";
import { routes } from "@/shared/config";
import { Appear, Button, EmptyState, ErrorNotice, SkeletonForm } from "@/shared/ui/kit";
import { useTrail } from "@/shared/lib/breadcrumbs";
import { PageHeader } from "@/shared/ui/page-header";

export const ID_PARAMETER = "id";

export type WebhookEditorScreenProps = { mode: "new" | "edit" };

export function WebhookEditorScreen({ mode }: WebhookEditorScreenProps) {
  const t = useTranslations();
  const trail = useTrail();
  const router = useRouter();
  const id = useSearchParams().get(ID_PARAMETER) ?? "";
  const webhooks = useWebhooks();
  const catalogue = useCatalogue();
  const scripts = useScripts();
  const named = webhooks.data?.data.webhooks.find((candidate) => candidate.id === id)?.title ?? id;
  const crumbs =
    mode === "new"
      ? trail.of(trail.section("webhooks"), { label: t("breadcrumbs.new.webhook") })
      : trail.of(trail.section("webhooks"), { label: named, href: routes.webhookDetails(id) }, { label: t("breadcrumbs.edit") });
  const header = <PageHeader breadcrumbs={crumbs} title={t(mode === "new" ? "webhooks.addTitle" : "webhooks.editTitle")} description={t("webhooks.formDescription")} />;
  const failure = webhooks.error ?? catalogue.error ?? scripts.error;
  if (!webhooks.data || !catalogue.data || !scripts.data) {
    return (
      <div className="grid gap-8">
        {header}
        {failure ? (
          <ErrorNotice
            title={t("errors.loadFailed")}
            description={failure.message}
            onRetry={() => {
              void webhooks.refetch();
              void catalogue.refetch();
              void scripts.refetch();
            }}
          />
        ) : (
          <SkeletonForm fields={6} />
        )}
      </div>
    );
  }
  const webhook = mode === "edit" ? (webhooks.data.data.webhooks.find((candidate) => candidate.id === id) ?? null) : null;
  if (mode === "edit" && !webhook) {
    return (
      <div className="grid gap-8">
        {header}
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
      </div>
    );
  }
  return (
    <div className="grid gap-8">
      {header}
      <Appear>
        <WebhookForm
          key={webhook?.id ?? "new"}
          webhook={webhook}
          revision={webhooks.data.revision}
          catalogue={catalogue.data}
          scripts={scripts.data}
          onSaved={() => router.push(routes.adminWebhooks)}
          onConflict={() => void webhooks.refetch()}
        />
      </Appear>
    </div>
  );
}
