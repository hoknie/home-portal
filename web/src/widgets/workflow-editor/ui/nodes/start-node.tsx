"use client";

import { Handle, type NodeProps, Position } from "@xyflow/react";
import { cn } from "cn";
import { History, Play } from "lucide-react";
import { useTranslations } from "next-intl";

import { inputNames } from "@/entities/workflow";

import { useEditor } from "../../model/editor-context";
import type { CanvasNode } from "./node-data";

export function StartNode({ selected }: NodeProps<CanvasNode>) {
  const t = useTranslations("workflowEditor");
  const editor = useEditor();
  const inputs = inputNames(editor.draft.inputs);
  const starters = editor.usedBy;
  const headerProblems = editor.problems.filter((problem) => problem.at !== "" && !problem.at.startsWith("steps")).length;
  return (
    <div
      role="group"
      aria-label={t("startNode")}
      className={cn(
        "flex size-full flex-col gap-1.5 rounded-2xl border border-border/70 bg-[color-mix(in_oklch,var(--color-primary)_7%,var(--glass-overlay-solid))] p-3 shadow-[0_1px_2px_rgb(0_0_0/0.06),0_4px_12px_-6px_rgb(0_0_0/0.12)]",
        selected && "outline-2 outline-offset-2 outline-primary/70",
      )}
    >
      <div className="flex items-center gap-2">
        <span data-start-icon="" className="grid size-7 shrink-0 place-items-center rounded-full bg-primary text-primary-foreground">
          <Play className="size-3.5" aria-hidden />
        </span>
        <span className="min-w-0 truncate text-sm font-semibold">{editor.draft.title || t("untitled")}</span>
        {headerProblems > 0 ? <span className="size-2 shrink-0 rounded-full bg-destructive" title={t("problemCount", { count: headerProblems })} /> : null}
        {editor.lastRunId ? (
          <button
            type="button"
            className="nodrag ms-auto rounded-md p-1 text-muted-foreground hover:bg-glass-tint hover:text-foreground"
            aria-label={t("run.showLast")}
            title={t("run.showLast")}
            onClick={(event) => {
              event.stopPropagation();
              editor.showRun(editor.lastRunId ?? "");
            }}
          >
            <History className="size-4" aria-hidden />
          </button>
        ) : null}
      </div>
      <p className="truncate text-xs text-muted-foreground">
        {starters.length === 0 ? t("onlyByHand") : t("startedBy", { names: starters.map((usage) => usage.title).join(", ") })}
      </p>
      <div className="flex min-w-0 flex-wrap gap-1 overflow-hidden">
        {inputs.length === 0 ? <span className="text-xs text-muted-foreground">{t("noWorkflowInputs")}</span> : null}
        {inputs.map((input) => (
          <span key={input} className="rounded-full bg-background/70 px-2 py-0.5 font-mono text-[11px]">
            {input}
          </span>
        ))}
      </div>
      <Handle type="source" position={Position.Bottom} className="!size-1 !min-w-0 !border-0 !bg-transparent" isConnectable={false} />
    </div>
  );
}
