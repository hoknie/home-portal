"use client";

import { useTranslations } from "next-intl";

import { type Run, useAutomations, useScripts } from "@/entities/automation";
import { useDashboard } from "@/entities/dashboard";
import { mayOpen, useSession } from "@/entities/session";
import { useWebhooks } from "@/entities/webhook";
import { useWorkflows } from "@/entities/workflow";
import { routes } from "@/shared/config";
import type { Reference } from "@/shared/ui/item-reference";

export const WORKFLOW_SOURCE = "workflow:";
export const WIDGET_SOURCE = "widget:";
export const WEBHOOK_FIELD = "webhook.id";

export type Target = { run: { script: string } | null; workflow: { id: string } | null };

export type ItemReferences = {
  automation: (id: string) => Reference;
  webhook: (id: string) => Reference;
  workflow: (id: string) => Reference;
  script: (path: string) => Reference;
  widget: (id: string) => Reference;
  sourceOf: (run: Run) => { reference: Reference; also: Reference | null };
  actionOf: (target: Target) => Reference | null;
};

export function useItemReferences(): ItemReferences {
  const t = useTranslations("references");
  const automations = useAutomations().data?.data.automations ?? [];
  const webhooks = useWebhooks().data?.data.webhooks ?? [];
  const workflows = useWorkflows().data?.data.workflows ?? [];
  const scripts = useScripts();
  const session = useSession();
  const widgets = useDashboard().data?.widgets ?? [];
  const scriptsOpen = (scripts.data?.editing ?? false) && mayOpen(session.data, "scripts");
  const automation = (id: string): Reference => {
    const found = automations.find((candidate) => candidate.id === id);
    return { kind: t("automation"), text: found?.title ?? id, href: found ? routes.editAutomation(id) : null };
  };
  const webhook = (id: string): Reference => {
    const found = webhooks.find((candidate) => candidate.id === id);
    return { kind: t("webhook"), text: found?.title ?? id, href: found ? routes.webhookDetails(id) : null };
  };
  const workflow = (id: string): Reference => {
    const found = workflows.find((candidate) => candidate.id === id);
    return { kind: t("workflow"), text: found?.title ?? id, href: found ? routes.workflow(id) : null };
  };
  const script = (path: string): Reference => ({ kind: t("script"), text: path, href: scriptsOpen ? routes.script(path) : null, mono: true });
  const widget = (id: string): Reference => {
    const found = widgets.find((candidate) => candidate.id === id);
    return { kind: t("widget"), text: found?.title ?? id, href: mayOpen(session.data, "layout") ? routes.layoutWidget(id) : null };
  };
  const sourceOf = (run: Run) => {
    if (run.automation.startsWith(WIDGET_SOURCE)) {
      return { reference: widget(run.automation.slice(WIDGET_SOURCE.length)), also: run.workflow ? workflow(run.workflow) : null };
    }
    if (run.automation.startsWith(WORKFLOW_SOURCE)) {
      return { reference: workflow(run.automation.slice(WORKFLOW_SOURCE.length)), also: null };
    }
    const reference = run.fields[WEBHOOK_FIELD] === run.automation ? webhook(run.automation) : automation(run.automation);
    return { reference, also: run.workflow ? workflow(run.workflow) : null };
  };
  const actionOf = (target: Target) => (target.workflow ? workflow(target.workflow.id) : target.run ? script(target.run.script) : null);
  return { automation, webhook, workflow, script, widget, sourceOf, actionOf };
}
