"use client";

import { BaseEdge, EdgeLabelRenderer, type EdgeProps, getSmoothStepPath } from "@xyflow/react";
import { cn } from "cn";
import { Plus } from "lucide-react";
import { useTranslations } from "next-intl";

import { useEditor } from "../../model/editor-context";
import { slotKey } from "../../model/edits/drop";
import type { CanvasEdge } from "./edge-data";

export const EDGE_TONE: Record<string, string> = { up: "!stroke-status-up", down: "!stroke-status-down", degraded: "!stroke-status-degraded" };

export function useEdgeLabel() {
  const t = useTranslations("workflowHelp.edges");
  return (label: NonNullable<CanvasEdge["data"]>["label"]) => (label ? t(label.key, label.params as Record<string, string | number>) : null);
}

export function FlowEdge({ id, sourceX, sourceY, targetX, targetY, sourcePosition, targetPosition, data, markerEnd }: EdgeProps<CanvasEdge>) {
  const t = useTranslations("workflowEditor");
  const editor = useEditor();
  const labelText = useEdgeLabel();
  const [path, , labelY] = getSmoothStepPath({ sourceX, sourceY, sourcePosition, targetX, targetY, targetPosition, borderRadius: 14 });
  const text = labelText(data?.label);
  const slot = data?.slot;
  return (
    <>
      <BaseEdge
        id={id}
        path={path}
        markerEnd={markerEnd}
        className={cn(
          "!stroke-muted-foreground/50 transition-[stroke,opacity]",
          data?.tone && `!stroke-[2.5px] ${EDGE_TONE[data.tone]}`,
          data?.taken && "!stroke-[3px]",
          data?.dimmed && "opacity-30",
        )}
      />
      <EdgeLabelRenderer>
        <div className="nodrag nopan pointer-events-auto absolute flex flex-col items-center gap-1" style={{ transform: `translate(-50%, -50%) translate(${targetX}px, ${text && !slot ? labelY : (sourceY + targetY) / 2}px)` }}>
          {text ? (
            <span className={cn("rounded-full border border-glass-edge bg-background px-2 py-0.5 text-[11px] font-medium text-muted-foreground shadow-xs", data?.taken && "border-primary text-primary")}>{text}</span>
          ) : null}
          {slot && !editor.readOnly ? (
            <button
              type="button"
              aria-label={t("insertHere")}
              data-slot={slotKey(slot)}
              onClick={() => editor.openPalette(slot)}
              className={cn(
                "grid place-items-center rounded-full border border-glass-edge bg-background text-muted-foreground shadow-xs transition-all hover:scale-110 hover:border-primary hover:text-primary",
                data?.dragging ? "size-9 border-2 border-dashed border-primary text-primary" : "size-6 opacity-70 hover:opacity-100",
              )}
            >
              <Plus className="size-3.5" aria-hidden />
            </button>
          ) : null}
        </div>
      </EdgeLabelRenderer>
    </>
  );
}

export function LoopEdge({ id, sourceX, sourceY, targetX, targetY, data, markerEnd }: EdgeProps<CanvasEdge>) {
  const labelText = useEdgeLabel();
  const right = Math.max(sourceX, targetX, data?.right ?? sourceX) + 20;
  const path = `M ${sourceX} ${sourceY} L ${right} ${sourceY} L ${right} ${targetY} L ${targetX} ${targetY}`;
  const text = labelText(data?.label);
  return (
    <>
      <BaseEdge id={id} path={path} markerEnd={markerEnd} className="!stroke-violet-500/60 [stroke-dasharray:6_4]" />
      {text ? (
        <EdgeLabelRenderer>
          <span
            className="absolute rounded-full border border-violet-500/40 bg-background px-2 py-0.5 text-[11px] text-violet-600 dark:text-violet-300"
            style={{ transform: `translate(-50%, -50%) translate(${right}px, ${(sourceY + targetY) / 2}px)` }}
          >
            {text}
          </span>
        </EdgeLabelRenderer>
      ) : null}
    </>
  );
}
