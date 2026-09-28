"use client";

import { cn } from "cn";
import { useTranslations } from "next-intl";

import type { TraceEntry } from "@/entities/automation";

const DOT: Record<string, string> = {
  running: "bg-status-degraded animate-pulse",
  succeeded: "bg-status-up",
  failed: "bg-status-down",
  "timed-out": "bg-status-down",
  stopped: "bg-muted-foreground",
  skipped: "bg-muted-foreground/50",
};

export function StepsStrip({ entries, onChoose }: { entries: TraceEntry[]; onChoose: (path: string) => void }) {
  const t = useTranslations("workflowEditor.run");
  return (
    <ol aria-label={t("steps")} className="flex max-w-full items-center gap-1 overflow-x-auto rounded-full border border-glass-edge bg-[var(--glass-overlay-solid)] p-1 shadow-lg">
      {entries.map((entry, index) => (
        <li key={`${index}-${entry.path}`} className="flex shrink-0 items-center gap-1">
          {index > 0 ? <span className="h-px w-3 bg-muted-foreground/40" aria-hidden /> : null}
          <button
            type="button"
            onClick={() => onChoose(entry.path)}
            data-outcome={entry.outcome}
            className="flex items-center gap-1.5 rounded-full px-2.5 py-1 text-xs hover:bg-accent"
          >
            <span className={cn("size-2 rounded-full", DOT[entry.outcome] ?? "bg-muted-foreground/50")} aria-hidden />
            <span className="max-w-32 truncate">{entry.label}</span>
            <span className="text-muted-foreground tabular-nums">{t("duration", { milliseconds: entry.duration_milliseconds })}</span>
          </button>
        </li>
      ))}
    </ol>
  );
}
