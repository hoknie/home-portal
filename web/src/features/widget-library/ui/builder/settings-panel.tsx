"use client";

import { Database, Eye, Puzzle, ShieldCheck } from "lucide-react";
import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import { cn } from "@/shared/lib/cn";

export type PanelTab = "block" | "data" | "look" | "access";

export const PANEL_TABS: { tab: PanelTab; icon: typeof Puzzle }[] = [
  { tab: "block", icon: Puzzle },
  { tab: "data", icon: Database },
  { tab: "look", icon: Eye },
  { tab: "access", icon: ShieldCheck },
];

export type SettingsPanelProps = {
  tab: PanelTab;
  failing: ReadonlySet<PanelTab>;
  onTab: (tab: PanelTab) => void;
  collapse: ReactNode;
  children: ReactNode;
};

export function SettingsPanel({ tab, failing, onTab, collapse, children }: SettingsPanelProps) {
  const t = useTranslations("widgetBuilder");
  return (
    <section className="glass-panel flex min-h-0 flex-1 flex-col overflow-hidden rounded-2xl" aria-label={t("panel.title")} data-settings-panel="">
      <div className="flex items-center gap-1 border-b border-glass-edge p-2">
        <div role="tablist" aria-label={t("panel.title")} className="grid flex-1 grid-cols-4 gap-1 rounded-xl bg-muted/50 p-1">
          {PANEL_TABS.map(({ tab: name, icon: Icon }) => {
            const chosen = name === tab;
            const bad = failing.has(name);
            return (
              <button
                key={name}
                type="button"
                role="tab"
                id={`builder-tab-${name}`}
                aria-selected={chosen}
                aria-controls="builder-tab-body"
                aria-label={bad ? t("panel.hasErrors", { tab: t(`tabs.${name}`) }) : t(`tabs.${name}`)}
                onClick={() => onTab(name)}
                className={cn(
                  "relative flex min-w-0 items-center justify-center gap-1.5 rounded-lg px-2 py-1.5 text-xs font-medium transition-colors outline-none focus-visible:ring-2 focus-visible:ring-ring",
                  chosen ? "bg-background text-foreground" : "text-muted-foreground hover:text-foreground",
                )}
              >
                <Icon className="size-3.5 shrink-0" aria-hidden />
                <span className="truncate">{t(`tabs.${name}`)}</span>
                {bad ? <span aria-hidden className="absolute top-1 right-1 size-1.5 rounded-full bg-status-degraded" /> : null}
              </button>
            );
          })}
        </div>
        {collapse}
      </div>
      <div id="builder-tab-body" role="tabpanel" aria-labelledby={`builder-tab-${tab}`} className="min-h-0 flex-1 overflow-y-auto p-4">
        {children}
      </div>
    </section>
  );
}
