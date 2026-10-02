"use client";

import { BaseEdge, EdgeLabelRenderer, type EdgeProps, getSmoothStepPath } from "@xyflow/react";
import { cn } from "cn";
import { Plus } from "lucide-react";
import { useTranslations } from "next-intl";

import { GAP_Y } from "@/entities/workflow";
import { BareButton, CANVAS } from "@/shared/ui/kit";

import { useEditor } from "../../model/editor-context";
import { slotKey } from "../../model/edits/drop";
import type { CanvasEdge } from "./edge-data";

export const EDGE_TONE: Record<string, string> = { up: "!stroke-status-up", down: "!stroke-status-down", degraded: "!stroke-status-degraded" };

export function useEdgeLabel() {
  const t = useTranslations("workflowHelp.edges");
  return (label: NonNullable<CanvasEdge["data"]>["label"]) => (label ? t(label.key, label.params as Record<string, string | number>) : null);
}

export const RADIUS = 14;

export const LOOP_CLEARANCE = 32;

export function railPath(source: { x: number; y: number }, target: { x: number; y: number }, rail: number) {
  const level = Math.min(Math.max(rail, source.y), target.y);
  const across = target.x - source.x;
  if (Math.abs(across) < 1) {
    return `M ${source.x} ${source.y} L ${target.x} ${target.y}`;
  }
  const direction = Math.sign(across);
  const radius = Math.max(0, Math.min(RADIUS, Math.abs(across) / 2, level - source.y, target.y - level));
  return [
    `M ${source.x} ${source.y}`,
    `L ${source.x} ${level - radius}`,
    `Q ${source.x} ${level} ${source.x + direction * radius} ${level}`,
    `L ${target.x - direction * radius} ${level}`,
    `Q ${target.x} ${level} ${target.x} ${level + radius}`,
    `L ${target.x} ${target.y}`,
  ].join(" ");
}

export function slotSpot(source: { x: number; y: number }, target: { x: number; y: number }, rail: number | undefined) {
  return rail === undefined ? { x: target.x, y: (source.y + target.y) / 2 } : { x: source.x, y: Math.min(source.y + GAP_Y / 2, Math.max(rail, source.y)) };
}

export function FlowEdge({ id, sourceX, sourceY, targetX, targetY, sourcePosition, targetPosition, data, markerEnd }: EdgeProps<CanvasEdge>) {
  const t = useTranslations("workflowEditor");
  const editor = useEditor();
  const labelText = useEdgeLabel();
  const [smooth, , labelY] = getSmoothStepPath({ sourceX, sourceY, sourcePosition, targetX, targetY, targetPosition, borderRadius: RADIUS });
  const rail = data?.rail;
  const path = rail === undefined ? smooth : railPath({ x: sourceX, y: sourceY }, { x: targetX, y: targetY }, rail);
  const text = labelText(data?.label);
  const slot = data?.slot;
  const spot = text && !slot ? { x: targetX, y: labelY } : slotSpot({ x: sourceX, y: sourceY }, { x: targetX, y: targetY }, rail);
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
          data?.dashed && "[stroke-dasharray:6_4] opacity-60",
        )}
      />
      <EdgeLabelRenderer>
        <div
          className="nodrag nopan pointer-events-auto absolute flex flex-col items-center gap-1"
          style={{ transform: `translate(-50%, -50%) translate(${spot.x}px, ${spot.y}px)` }}
        >
          {text ? (
            <span
              className={cn(
                CANVAS.edgeLabel,
                "px-2 py-0.5 text-xs font-medium text-muted-foreground",
                data?.taken && "border-primary text-primary",
              )}
            >
              {text}
            </span>
          ) : null}
          {slot && !editor.readOnly ? (
            <BareButton
              aria-label={t("insertHere")}
              data-slot={slotKey(slot)}
              onClick={() => editor.openPalette(slot)}
              className={cn(
                CANVAS.edgeLabel,
                "grid place-items-center text-muted-foreground transition-all hover:scale-110 hover:border-primary hover:text-primary",
                data?.dragging ? "size-10 border-2 border-dashed border-primary text-primary" : "size-8 opacity-70 hover:opacity-100",
              )}
            >
              <Plus className="size-4" aria-hidden />
            </BareButton>
          ) : null}
        </div>
      </EdgeLabelRenderer>
    </>
  );
}

export function LoopEdge({ id, sourceX, sourceY, targetX, targetY, data, markerEnd }: EdgeProps<CanvasEdge>) {
  const labelText = useEdgeLabel();
  const right = Math.max(sourceX, targetX, data?.right ?? sourceX) + LOOP_CLEARANCE;
  const path = `M ${sourceX} ${sourceY} L ${right} ${sourceY} L ${right} ${targetY} L ${targetX} ${targetY}`;
  const text = labelText(data?.label);
  return (
    <>
      <BaseEdge id={id} path={path} markerEnd={markerEnd} className="!stroke-palette-violet/60 [stroke-dasharray:6_4]" />
      {text ? (
        <EdgeLabelRenderer>
          <span
            className="absolute rounded-full border border-palette-violet/40 bg-background px-2 py-0.5 text-xs text-palette-violet"
            style={{ transform: `translate(-50%, -50%) translate(${right}px, ${(sourceY + targetY) / 2}px)` }}
          >
            {text}
          </span>
        </EdgeLabelRenderer>
      ) : null}
    </>
  );
}
