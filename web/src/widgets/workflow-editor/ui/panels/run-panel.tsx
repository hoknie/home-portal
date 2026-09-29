"use client";

import { cn } from "cn";
import { List } from "lucide-react";
import { useTranslations } from "next-intl";
import { type ReactNode, useState } from "react";

import { StopRunButton } from "@/features/stop-run";
import { OutcomeBadge, type Run, type TraceEntry, TraceEntryView, depthOf, isActive, passRows, rowHeading } from "@/entities/automation";
import { START_ID } from "@/entities/workflow";
import { AddressLink } from "@/shared/ui/address-link";
import { buttonVariants } from "@/shared/ui/primitives";

import { useEditor } from "../../model/editor-context";
import { CloseLink } from "./side-column";

const DOT: Record<string, string> = {
  running: "bg-status-degraded animate-pulse",
  succeeded: "bg-status-up",
  failed: "bg-status-down",
  "timed-out": "bg-status-down",
  stopped: "bg-muted-foreground",
  skipped: "bg-muted-foreground/50",
};

const INDENT_REM = 1;

export function entryFor(entries: TraceEntry[], chosen: number | null, selected: string | null): number | null {
  if (chosen !== null && entries[chosen]?.path === selected) {
    return chosen;
  }
  if (selected === null) {
    return null;
  }
  const own = entries.map((entry, index) => ({ entry, index })).filter(({ entry }) => entry.path === selected);
  return (own.find(({ entry }) => entry.outcome === "failed" || entry.outcome === "timed-out") ?? own.at(-1))?.index ?? null;
}

function RunSteps({ entries, current, onChoose }: { entries: TraceEntry[]; current: number | null; onChoose: (index: number) => void }) {
  const t = useTranslations("workflowEditor.run");
  const trace = useTranslations("workflows.trace");
  const rows = passRows(entries);
  const orders = rows.map((row, position) => rows.slice(0, position + 1).filter((earlier) => earlier.type === "entry").length);
  return (
    <ol aria-label={t("steps")} className="grid max-h-64 gap-0.5 overflow-y-auto">
      {rows.map((row, position) => {
        if (row.type !== "entry") {
          return (
            <li key={`${row.type}-${position}`} className="px-2 pt-1 text-[11px] font-medium text-muted-foreground" style={{ paddingInlineStart: `${row.depth * INDENT_REM + 0.5}rem` }}>
              {rowHeading(row, trace)}
            </li>
          );
        }
        const { entry, index } = row;
        return (
          <li key={`${index}-${entry.path}`}>
            <button
              type="button"
              onClick={() => onChoose(index)}
              data-outcome={entry.outcome}
              aria-current={index === current ? "step" : undefined}
              style={{ paddingInlineStart: `${depthOf(entry.path) * INDENT_REM + 0.5}rem` }}
              className={cn("grid w-full grid-cols-[auto_auto_1fr_auto] items-center gap-2 rounded-md py-1 pe-2 text-start text-xs hover:bg-accent", index === current && "bg-accent")}
            >
              <span className="w-5 text-end text-muted-foreground tabular-nums">{orders[position]}</span>
              <span className={cn("size-2 rounded-full", DOT[entry.outcome] ?? "bg-muted-foreground/50")} aria-hidden />
              <span className="truncate">{entry.label}</span>
              <span className="text-muted-foreground tabular-nums">{t("duration", { milliseconds: entry.duration_milliseconds })}</span>
            </button>
          </li>
        );
      })}
    </ol>
  );
}

function StepDetails({ entries, current, selected, onChoose }: { entries: TraceEntry[]; current: number | null; selected: string | null; onChoose: (index: number) => void }) {
  const t = useTranslations("workflowEditor.run");
  if (selected === null) {
    return <p className="text-sm text-muted-foreground">{t("chooseStep")}</p>;
  }
  if (current === null) {
    return <p className="text-sm text-muted-foreground">{t("notReached")}</p>;
  }
  const passes = entries.map((entry, index) => ({ entry, index })).filter(({ entry }) => entry.path === selected);
  return (
    <div className="grid gap-3">
      {passes.length > 1 ? (
        <div role="group" aria-label={t("passes")} className="flex flex-wrap gap-1">
          {passes.map(({ entry, index }, number) => (
            <button
              key={index}
              type="button"
              aria-pressed={index === current}
              onClick={() => onChoose(index)}
              className={cn(
                "flex items-center gap-1 rounded-md border border-glass-edge px-2 py-0.5 text-xs tabular-nums",
                index === current ? "bg-primary text-primary-foreground" : "text-muted-foreground hover:text-foreground",
              )}
            >
              <span className={cn("size-1.5 rounded-full", DOT[entry.outcome] ?? "bg-muted-foreground/50")} aria-hidden />
              {number + 1}
            </button>
          ))}
        </div>
      ) : null}
      <ol>
        <TraceEntryView entry={entries[current]} />
      </ol>
    </div>
  );
}

export type RunPanelProps = { run: Run | null; title: string; stale: boolean; missing: boolean; historyHref: string; closeHref: string; empty: ReactNode };

export function RunPanel({ run, title, stale, missing, historyHref, closeHref, empty }: RunPanelProps) {
  const t = useTranslations("workflowEditor.run");
  const editor = useEditor();
  const [chosen, setChosen] = useState<number | null>(null);
  const entries = run?.trace?.entries ?? [];
  const selected = editor.selected && editor.selected !== START_ID ? editor.selected : null;
  const current = entryFor(entries, chosen, selected);
  const choose = (index: number) => {
    setChosen(index);
    editor.reveal(entries[index].path);
  };
  return (
    <aside aria-label={t("title")} className="glass-panel flex max-h-[45%] min-h-0 w-full shrink-0 flex-col overflow-hidden rounded-2xl md:max-h-full md:w-[26rem]">
      <header className="grid gap-2 border-b border-glass-edge p-4">
        <div className="flex flex-wrap items-center gap-2 text-sm">
          <h2 className="text-base font-semibold">{t("title")}</h2>
          {run ? <OutcomeBadge outcome={run.outcome.result} /> : null}
          {run && !isActive(run) ? <span className="text-xs text-muted-foreground tabular-nums">{t("duration", { milliseconds: run.outcome.duration_milliseconds })}</span> : null}
          <span className="ms-auto flex items-center gap-1">
            {run && isActive(run) ? <StopRunButton run={run} title={title} /> : null}
            <AddressLink href={historyHref} className={cn(buttonVariants({ variant: "ghost", size: "sm" }))}>
              <List aria-hidden />
              {t("allRuns")}
            </AddressLink>
            <CloseLink href={closeHref} />
          </span>
        </div>
        {stale ? <p className="text-xs text-status-degraded">{t("stale")}</p> : null}
      </header>
      {run === null ? (
        <div role="status" className="grid gap-3 p-4 text-sm text-muted-foreground">
          {missing ? <p>{t("notFound")}</p> : empty}
        </div>
      ) : (
        <div className="grid min-h-0 flex-1 grid-rows-[auto_minmax(0,1fr)] overflow-hidden">
          <section className="border-b border-glass-edge p-2">
            <RunSteps entries={entries} current={current} onChoose={choose} />
          </section>
          <section aria-label={t("stepDetails")} className="min-h-0 overflow-y-auto p-4">
            <StepDetails entries={entries} current={current} selected={selected} onChoose={choose} />
          </section>
        </div>
      )}
    </aside>
  );
}
