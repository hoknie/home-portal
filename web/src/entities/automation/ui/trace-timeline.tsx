"use client";

import { LoaderCircle } from "lucide-react";
import { useTranslations } from "next-intl";

import { JsonView } from "@/shared/ui/json-view";

import type { StepOutcome, Trace, TraceEntry } from "../model/schema";
import { terminalText } from "../model/terminal-text";

const MARK: Record<StepOutcome, string> = {
  running: "bg-status-degraded",
  succeeded: "bg-status-up",
  failed: "bg-status-down",
  "timed-out": "bg-status-down",
  stopped: "bg-muted-foreground",
  skipped: "bg-muted-foreground/50",
  unknown: "bg-muted-foreground/50",
};

export function depthOf(path: string) {
  return Math.max(0, (path.match(/\[\d+\]/g)?.length ?? 1) - 1);
}

export function answerOf(entry: TraceEntry): { value: unknown; shortened: boolean } | null {
  for (const [text, shortened] of [
    [entry.output, false],
    [entry.shape, true],
  ] as const) {
    if (text) {
      try {
        return { value: JSON.parse(text) as unknown, shortened: shortened && entry.shape !== entry.output };
      } catch {
        continue;
      }
    }
  }
  return null;
}

function Output({ entry }: { entry: TraceEntry }) {
  const t = useTranslations("workflows.trace");
  const answer = entry.kind === "http" ? answerOf(entry) : null;
  const frame = "mt-1 max-h-72 overflow-auto rounded-md border border-glass-edge bg-glass-tint p-2";
  if (answer) {
    return (
      <div className={frame}>
        {answer.shortened ? <p className="mb-1 text-muted-foreground">{t("shortened")}</p> : null}
        <JsonView value={answer.value} label={t("answer")} />
      </div>
    );
  }
  return <pre className={`${frame} font-mono whitespace-pre-wrap break-all`}>{terminalText(entry.output ?? "")}</pre>;
}

function Entry({ entry }: { entry: TraceEntry }) {
  const t = useTranslations("workflows.trace");
  const outcome = t(`outcomes.${entry.outcome.replace("-", "_")}` as "outcomes.running");
  return (
    <li className="grid gap-1 py-1.5" style={{ paddingInlineStart: `${depthOf(entry.path) * 1.25}rem` }} data-outcome={entry.outcome}>
      <div className="flex min-w-0 flex-wrap items-center gap-x-2 gap-y-0.5 text-sm">
        {entry.outcome === "running" ? (
          <LoaderCircle className="size-3 animate-spin text-status-degraded" aria-hidden />
        ) : (
          <span className={`size-2.5 shrink-0 rounded-full ${MARK[entry.outcome]}`} aria-hidden />
        )}
        <span className="sr-only">{outcome}</span>
        <span className="font-medium">{entry.label}</span>
        <span className="font-mono text-xs text-muted-foreground">{entry.kind}</span>
        {entry.iteration !== null ? <span className="text-xs text-muted-foreground">{t("iteration", { number: entry.iteration + 1 })}</span> : null}
        <span className="ms-auto text-xs text-muted-foreground tabular-nums">{t("duration", { value: entry.duration_milliseconds })}</span>
      </div>
      {entry.detail ? <p className="text-xs break-all text-muted-foreground">{entry.detail}</p> : null}
      {entry.output || entry.shape ? (
        <details className="text-xs">
          <summary className="cursor-pointer text-muted-foreground">{t(entry.kind === "http" ? "answer" : "output")}</summary>
          <Output entry={entry} />
        </details>
      ) : null}
    </li>
  );
}

export function TraceTimeline({ trace }: { trace: Trace }) {
  const t = useTranslations("workflows.trace");
  return (
    <div className="grid gap-1">
      <p className="text-sm font-medium">{t("title")}</p>
      {trace.entries.length === 0 ? <p className="text-xs text-muted-foreground">{t("empty")}</p> : null}
      <ol className="divide-y divide-glass-edge" aria-label={t("title")}>
        {trace.entries.map((entry, index) => (
          <Entry key={`${index}-${entry.path}`} entry={entry} />
        ))}
      </ol>
      {trace.dropped > 0 ? <p className="text-xs text-muted-foreground">{t("dropped", { count: trace.dropped })}</p> : null}
    </div>
  );
}
