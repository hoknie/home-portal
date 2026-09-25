"use client";

import { SearchX } from "lucide-react";
import Link from "next/link";
import { useRouter, useSearchParams } from "next/navigation";
import { useTranslations } from "next-intl";

import { AutomationBuilder } from "@/features/automation-builder";
import { RunAutomationButton } from "@/features/run-automation";
import { useAutomations, useCatalogue, useScripts } from "@/entities/automation";
import { routes } from "@/shared/config";
import { EmptyState } from "@/shared/ui/empty-state";
import { ErrorNotice } from "@/shared/ui/error-notice";
import { PageHeader } from "@/shared/ui/page-header";
import { Button, Skeleton } from "@/shared/ui/primitives";

export const ID_PARAMETER = "id";

export type AutomationEditorScreenProps = { mode: "new" | "edit" };

export function AutomationEditorScreen({ mode }: AutomationEditorScreenProps) {
  const t = useTranslations();
  const router = useRouter();
  const id = useSearchParams().get(ID_PARAMETER) ?? "";
  const automations = useAutomations();
  const catalogue = useCatalogue();
  const scripts = useScripts();
  const all = automations.data?.data.automations ?? [];
  const automation = mode === "edit" ? (all.find((candidate) => candidate.id === id) ?? null) : null;
  const title = t(mode === "new" ? "automationBuilder.addTitle" : "automationBuilder.editTitle");
  const actions = automation ? <RunAutomationButton automation={automation} labelled /> : undefined;
  const header = <PageHeader title={title} description={t("automationBuilder.description")} actions={actions} />;
  const failure = automations.error ?? catalogue.error ?? scripts.error;
  if (!automations.data || !catalogue.data || !scripts.data) {
    return (
      <div className="grid gap-8">
        {header}
        {failure ? (
          <ErrorNotice
            title={t("errors.loadFailed")}
            description={failure.message}
            onRetry={() => {
              void automations.refetch();
              void catalogue.refetch();
              void scripts.refetch();
            }}
          />
        ) : (
          <Skeleton className="h-96 w-full" aria-busy="true" />
        )}
      </div>
    );
  }
  if (mode === "edit" && !automation) {
    return (
      <div className="grid gap-8">
        {header}
        <EmptyState
          icon={SearchX}
          title={t("automationBuilder.notFound")}
          description={t("automationBuilder.notFoundHint")}
          action={
            <Button asChild variant="outline">
              <Link href={routes.adminAutomations}>{t("automationBuilder.backToAutomations")}</Link>
            </Button>
          }
        />
      </div>
    );
  }
  return (
    <div className="grid gap-8">
      {header}
      <AutomationBuilder
        key={automation?.id ?? "new"}
        automation={automation}
        revision={automations.data.revision}
        taken={all.map((candidate) => candidate.id).filter((candidate) => candidate !== automation?.id)}
        catalogue={catalogue.data}
        scripts={scripts.data}
        onSaved={() => router.push(routes.adminAutomations)}
        onConflict={() => void automations.refetch()}
      />
    </div>
  );
}
