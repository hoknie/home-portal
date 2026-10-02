"use client";

import { LoaderCircle } from "lucide-react";
import { useTranslations } from "next-intl";

import { JsonView } from "@/shared/ui/json-view";

import type { StepOutcome, Trace, TraceEntry } from "../../model/schema";
import { hasScriptLog } from "../../model/script-log";
import { terminalText } from "../../model/terminal-text";
import { type TraceRow, depthOf, hasLog, passRows } from "./pass-groups";
import { ScriptLogDialog } from "./script-log-dialog";
import { LevelMark, StepLog } from "./step-log";

export { depthOf };

export const FAILED: StepOutcome[] = ["failed", "timed-out"];

const MARK: Record<StepOutcome, string> = {
  running: "bg-status-degraded",
  succeeded: "bg-status-up",
  failed: "bg-status-down",
  "timed-out": "bg-status-down",
  stopped: "bg-muted-foreground",
  skipped: "bg-muted-foreground/50",
  unknown: "bg-muted-foreground/50",
};

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

export function TraceEntryView({ entry, indent = true }: { entry: TraceEntry; indent?: boolean }) {
  const t = useTranslations("workflows.trace");
  const outcome = t(`outcomes.${entry.outcome.replace("-", "_")}` as "outcomes.running");
  return (
    <li className="grid gap-1 py-1.5" style={indent ? { paddingInlineStart: `${depthOf(entry.path) * 1.25}rem` } : undefined} data-outcome={entry.outcome}>
      <div className="flex min-w-0 flex-wrap items-center gap-x-2 gap-y-0.5 text-sm">
        {entry.outcome === "running" ? (
          <LoaderCircle className="size-3 animate-spin text-status-degraded" aria-hidden />
        ) : (
          <span className={`size-2.5 shrink-0 rounded-full ${MARK[entry.outcome]}`} aria-hidden />
        )}
        <span className="sr-only">{outcome}</span>
        <span className="font-medium">{entry.label}</span>
        <span className="font-mono text-xs text-muted-foreground">{entry.kind}</span>
        {entry.level ? <LevelMark level={entry.level} /> : null}
        {entry.iteration !== null ? <span className="text-xs text-muted-foreground">{t("iteration", { number: entry.iteration + 1 })}</span> : null}
        <span className="ms-auto text-xs text-muted-foreground tabular-nums">{t("duration", { value: entry.duration_milliseconds })}</span>
      </div>
      {entry.detail ? (
        <p className="truncate text-xs text-muted-foreground" title={entry.detail} data-detail="">
          {entry.detail}
        </p>
      ) : null}
      {hasLog(entry) ? (
        <details className="text-xs" open={FAILED.includes(entry.outcome)}>
          <summary className="cursor-pointer text-muted-foreground">{t("details")}</summary>
          <div className="mt-1 rounded-md border border-glass-edge bg-glass-tint p-2">
            <StepLog entry={entry} />
          </div>
        </details>
      ) : null}
      {hasScriptLog(entry) ? <ScriptLogDialog entry={entry} /> : null}
      {entry.kind !== "script" && (entry.output || entry.shape) ? (
        <details className="text-xs">
          <summary className="cursor-pointer text-muted-foreground">{t(entry.kind === "http" ? "answer" : "output")}</summary>
          <Output entry={entry} />
        </details>
      ) : null}
    </li>
  );
}

export function rowHeading(row: Exclude<TraceRow, { type: "entry" }>, t: (key: "pass" | "passWithItem" | "branch", values: Record<string, string | number>) => string) {
  if (row.type === "branch") {
    return t("branch", { number: row.number });
  }
  return row.item === null ? t("pass", { number: row.number }) : t("passWithItem", { number: row.number, item: row.item });
}

function Heading({ row }: { row: Exclude<TraceRow, { type: "entry" }> }) {
  const t = useTranslations("workflows.trace");
  return (
    <li className="py-1 text-xs font-medium text-muted-foreground" style={{ paddingInlineStart: `${row.depth * 1.25}rem` }} data-pass={row.type === "pass" ? row.number : undefined} data-branch={row.type === "branch" ? row.number : undefined}>
      {rowHeading(row, t)}
    </li>
  );
}

export function TraceTimeline({ trace, indent = true }: { trace: Trace; indent?: boolean }) {
  const t = useTranslations("workflows.trace");
  return (
    <div className="grid gap-1">
      <p className="text-sm font-medium">{t("title")}</p>
      {trace.entries.length === 0 ? <p className="text-xs text-muted-foreground">{t("empty")}</p> : null}
      <ol className="divide-y divide-glass-edge" aria-label={t("title")}>
        {passRows(trace.entries).map((row, index) => (row.type === "entry" ? <TraceEntryView key={`${row.index}-${row.entry.path}`} entry={row.entry} indent={indent} /> : <Heading key={`${row.type}-${"loop" in row ? row.loop : row.parallel}-${row.number}-${index}`} row={row} />))}
      </ol>
      {trace.dropped > 0 ? <p className="text-xs text-muted-foreground">{t("dropped", { count: trace.dropped })}</p> : null}
      {trace.outputs ? (
        <section className="mt-2 grid gap-1" aria-label={t("outputs")} data-trace-outputs="">
          <p className="text-sm font-medium">{t("outputs")}</p>
          <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 text-sm">
            {Object.entries(trace.outputs).map(([name, value]) => (
              <div key={name} className="contents">
                <dt className="font-mono text-muted-foreground">{name}</dt>
                <dd className="min-w-0 font-mono break-all">{typeof value === "string" ? value : JSON.stringify(value)}</dd>
              </div>
            ))}
          </dl>
        </section>
      ) : null}
    </div>
  );
}
