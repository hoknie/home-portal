"use client";

import { Handle, type NodeProps, Position } from "@xyflow/react";
import { cn } from "cn";
import { CircleAlert, TriangleAlert } from "lucide-react";
import { useTranslations } from "next-intl";

import { operatorName, parsePath } from "@/entities/workflow";

import { useEditor } from "../../model/editor-context";
import { summaryOf } from "../../model/summary";
import { FALLBACK_ICON, GROUP_TONE, KIND_ICONS, OUTCOME_DOT, OUTCOME_TINT } from "./kind-style";
import type { CanvasNode } from "./node-data";
import { NodeMenu } from "./node-menu";

const HIDDEN_HANDLE = "!size-1 !min-w-0 !border-0 !bg-transparent";

export function StepNode({ id, data, selected }: NodeProps<CanvasNode>) {
  const t = useTranslations("workflowEditor");
  const help = useTranslations("workflowHelp");
  const editor = useEditor();
  const operators = useTranslations("workflowEditor.condition.operators");
  const step = data.node.step;
  const path = data.node.path;
  if (!step || !path) {
    return null;
  }
  const kind = editor.kindOf(step.kind);
  const tone = GROUP_TONE[kind?.group ?? "actions"];
  const Icon = KIND_ICONS[step.kind] ?? FALLBACK_ICON;
  const own = editor.problems.filter((problem) => {
    const { path: at } = parsePath(problem.at);
    return at.length === path.length && at.every((place, index) => place.list === path[index].list && place.index === path[index].index);
  });
  const errors = own.filter((problem) => problem.severity === "error").length;
  const warnings = own.length - errors;
  const run = editor.overlay.get(id);
  const summary = summaryOf(step, editor.sources.workflows, editor.sources.automations, (operator) =>
    operators.has(operatorName(operator) as "equals") ? operators(operatorName(operator) as "equals") : operator,
  );
  const summaryText = "text" in summary ? summary.text : t(summary.key as "summaries.wait", summary.params);
  const kindName = help.has(`kinds.${step.kind}.name` as "kinds.if.name") ? help(`kinds.${step.kind}.name` as "kinds.if.name") : step.kind;
  return (
    <div
      role="group"
      aria-label={step.label ?? step.id}
      data-path={id}
      data-outcome={run?.running ? "running" : run?.outcome}
      data-dimmed={data.path?.dimmed || undefined}
      data-order={data.path?.order}
      className={cn(
        "group relative flex size-full rounded-xl border border-border/70 bg-[var(--glass-overlay-solid)] text-card-foreground shadow-[0_1px_2px_rgb(0_0_0/0.06),0_4px_12px_-6px_rgb(0_0_0/0.12)] transition-[box-shadow,opacity] duration-200 hover:shadow-[0_1px_2px_rgb(0_0_0/0.08),0_8px_20px_-8px_rgb(0_0_0/0.2)]",
        selected && "outline-2 outline-offset-2 outline-primary/70",
        run && OUTCOME_TINT[run.running ? "running" : run.outcome],
        data.path?.dimmed && "opacity-35",
      )}
    >
      {data.path?.order !== undefined ? (
        <span
          className={cn(
            "absolute -top-2.5 -left-2.5 z-10 grid size-5 place-items-center rounded-full text-[10px] font-semibold text-white ring-2 ring-background",
            run?.running ? "bg-status-degraded" : run && (run.outcome === "failed" || run.outcome === "timed-out") ? "bg-status-down" : "bg-status-up",
          )}
          aria-label={t("run.order", { number: data.path.order })}
        >
          {data.path.order}
        </span>
      ) : null}
      <Handle type="target" position={Position.Top} className={HIDDEN_HANDLE} isConnectable={false} />
      {step.kind === "loop" ? <Handle id="again" type="target" position={Position.Right} className={HIDDEN_HANDLE} isConnectable={false} /> : null}
      <div className="grid min-w-0 flex-1 content-center gap-1.5 py-2.5 ps-3 pe-1.5">
        <div className="flex min-w-0 items-center gap-2">
          <span className={cn("grid size-8 shrink-0 place-items-center rounded-lg", tone.badge)}>
            <Icon className="size-4" aria-hidden />
          </span>
          <div className="grid min-w-0">
            <span className="truncate text-sm leading-tight font-semibold">{step.label ?? step.id}</span>
            <span className="truncate text-[11px] tracking-wide text-muted-foreground uppercase">{kindName}</span>
          </div>
          <span className="ms-auto flex shrink-0 items-center gap-0.5">
            {errors > 0 ? (
              <span className="flex items-center gap-0.5 text-xs text-destructive" title={t("problemCount", { count: errors })}>
                <CircleAlert className="size-3.5" aria-hidden />
                {errors}
              </span>
            ) : null}
            {warnings > 0 ? (
              <span className="flex items-center gap-0.5 text-xs text-status-degraded" title={t("warningCount", { count: warnings })}>
                <TriangleAlert className="size-3.5" aria-hidden />
                {warnings}
              </span>
            ) : null}
            <NodeMenu path={path} step={step} />
          </span>
        </div>
        <div className="flex min-w-0 items-center gap-2">
          <span className="min-w-0 flex-1 truncate font-mono text-[11px] text-muted-foreground">{summaryText}</span>
          {run ? (
            <span className="flex shrink-0 items-center gap-1 rounded-full bg-muted px-1.5 text-[11px] tabular-nums text-muted-foreground">
              <span className={cn("size-1.5 rounded-full", OUTCOME_DOT[run.running ? "running" : run.outcome])} aria-hidden />
              {run.passes > 1 ? t("run.passes", { count: run.passes, milliseconds: run.durationMilliseconds }) : t("run.duration", { milliseconds: run.durationMilliseconds })}
            </span>
          ) : null}
        </div>
      </div>
      <Handle type="source" position={Position.Bottom} className={HIDDEN_HANDLE} isConnectable={false} />
    </div>
  );
}
