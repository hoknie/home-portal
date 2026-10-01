"use client";

import { cn } from "cn";
import { ChevronDown, List } from "lucide-react";
import { useTranslations } from "next-intl";
import { type ReactNode, useEffect, useRef, useState } from "react";

import { StopRunButton } from "@/features/runs/stop-run";
import { OutcomeBadge, type Run, type TraceEntry, TraceEntryView, depthOf, isActive, passRows, rowHeading } from "@/entities/automation";
import { START_ID, at, parsePath } from "@/entities/workflow";
import { AddressLink } from "@/shared/ui/address-link";
import { buttonVariants } from "@/shared/ui/primitives";

import { PANEL_WIDTH_CLASS } from "../resizing/panel-row";
import { useEditor } from "../../model/editor-context";
import { StepCard } from "../inspector/step-card";
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
export const SCROLL_SLACK_PIXELS = 4;

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
  const list = useRef<HTMLOListElement>(null);
  const [more, setMore] = useState(false);
  const measure = () => {
    const element = list.current;
    setMore(element !== null && element.scrollHeight - element.scrollTop - element.clientHeight > SCROLL_SLACK_PIXELS);
  };
  useEffect(() => {
    const element = list.current;
    if (element === null) {
      return;
    }
    const watcher = new ResizeObserver(() => measure());
    watcher.observe(element);
    return () => watcher.disconnect();
  }, []);
  return (
    <div className="relative">
    <ol ref={list} onScroll={measure} aria-label={t("steps")} className="grid max-h-64 gap-0.5 overflow-y-auto">
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
      {more ? (
        <div className="pointer-events-none absolute inset-x-0 bottom-0 flex h-10 items-end justify-center bg-gradient-to-t from-[var(--glass-overlay-solid)] to-transparent">
          <button
            type="button"
            className="pointer-events-auto mb-0.5 flex items-center gap-1 rounded-full border border-glass-edge bg-background px-2 py-0.5 text-[11px] text-muted-foreground shadow-sm hover:text-foreground"
            onClick={() => list.current?.scrollTo({ top: list.current.scrollHeight, behavior: "smooth" })}
          >
            <ChevronDown className="size-3" aria-hidden />
            {t("moreBelow")}
          </button>
        </div>
      ) : null}
    </div>
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
        <TraceEntryView entry={entries[current]} indent={false} />
      </ol>
      <StepSettings path={selected} />
    </div>
  );
}

function StepSettings({ path }: { path: string }) {
  const t = useTranslations("workflowEditor.card");
  const editor = useEditor();
  const step = at(editor.draft.steps, parsePath(path).path);
  if (!step) {
    return null;
  }
  return (
    <details className="text-xs">
      <summary className="cursor-pointer text-muted-foreground">{t("settings")}</summary>
      <div className="mt-1 rounded-md border border-glass-edge bg-glass-tint p-2">
        <StepCard step={step} compact />
      </div>
    </details>
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
    <aside aria-label={t("title")} className={cn("glass-panel flex max-h-[45%] min-h-0 w-full shrink-0 flex-col overflow-hidden rounded-2xl md:max-h-full", PANEL_WIDTH_CLASS)}>
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
          <section className="grid gap-1 p-2">
            <h3 className="px-2 text-[11px] font-semibold tracking-wide text-muted-foreground uppercase">{t("stepsCount", { count: entries.length })}</h3>
            <RunSteps entries={entries} current={current} onChoose={choose} />
          </section>
          <section aria-label={t("stepDetails")} className="flex min-h-0 flex-col border-t-2 border-glass-edge bg-glass-tint/60">
            <h3 className="flex min-w-0 items-center gap-1 border-b border-glass-edge px-4 py-2 text-[11px] font-semibold tracking-wide text-muted-foreground uppercase">
              {t("stepDetails")}
              {current !== null ? <span className="truncate font-normal tracking-normal normal-case text-foreground before:me-1 before:text-muted-foreground before:content-['·']">{entries[current].label}</span> : null}
            </h3>
            <div className="min-h-0 flex-1 overflow-y-auto p-4">
              <StepDetails entries={entries} current={current} selected={selected} onChoose={choose} />
            </div>
          </section>
        </div>
      )}
    </aside>
  );
}
