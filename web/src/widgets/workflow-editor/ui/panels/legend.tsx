"use client";

import { ArrowDown, Braces, LayoutGrid, Play, Plus } from "lucide-react";
import { useTranslations } from "next-intl";

import { Button, Heading, Panel } from "@/shared/ui/kit";

export const LEGEND_KEY = "home-portal.workflow-editor.legend-dismissed";

export function legendDismissed() {
  try {
    return window.localStorage.getItem(LEGEND_KEY) === "1";
  } catch {
    return false;
  }
}

export function rememberLegend(dismissed: boolean) {
  try {
    if (dismissed) {
      window.localStorage.setItem(LEGEND_KEY, "1");
    } else {
      window.localStorage.removeItem(LEGEND_KEY);
    }
  } catch {
    return;
  }
}

const ITEMS = [
  { key: "start", icon: Play },
  { key: "arrows", icon: ArrowDown },
  { key: "plus", icon: Plus },
  { key: "palette", icon: LayoutGrid },
  { key: "completion", icon: Braces },
] as const;

export function Legend({ onDismiss }: { onDismiss: () => void }) {
  const t = useTranslations("workflowHelp.legend");
  return (
    <Panel padding="none" aria-label={t("title")} className="grid w-80 max-w-[calc(100vw-2rem)] gap-3 rounded-2xl p-4">
      <Heading level="group" as="h2" className="text-sm font-semibold">{t("title")}</Heading>
      <ul className="grid gap-2">
        {ITEMS.map(({ key, icon: Icon }) => (
          <li key={key} className="flex gap-2 text-xs text-muted-foreground">
            <Icon className="mt-0.5 size-3.5 shrink-0 text-primary" aria-hidden />
            {t(key)}
          </li>
        ))}
      </ul>
      <Button type="button" size="sm" className="w-fit" onClick={onDismiss}>
        {t("dismiss")}
      </Button>
    </Panel>
  );
}
