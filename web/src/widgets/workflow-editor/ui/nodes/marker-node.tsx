"use client";

import { Handle, type NodeProps, Position } from "@xyflow/react";
import { cn } from "cn";
import { CircleCheck, CircleX, LogOut, SkipForward } from "lucide-react";
import { useTranslations } from "next-intl";

import type { CanvasNode } from "./node-data";

const LOOK = {
  succeeded: { icon: CircleCheck, tone: "border-status-up/50 text-status-up" },
  failed: { icon: CircleX, tone: "border-status-down/50 text-status-down" },
  break: { icon: LogOut, tone: "border-palette-violet/50 text-palette-violet" },
  continue: { icon: SkipForward, tone: "border-palette-violet/50 text-palette-violet" },
} as const;

export function MarkerNode({ data }: NodeProps<CanvasNode>) {
  const t = useTranslations("workflowEditor.markers");
  const closing = data.node.closing ?? "succeeded";
  const { icon: Icon, tone } = LOOK[closing];
  return (
    <div
      role="note"
      data-closing={closing}
      className={cn(
        "flex size-full items-center justify-center gap-1.5 rounded-full border-2 border-dashed bg-background text-xs font-medium transition-opacity",
        tone,
        (data.node.unreachable || data.path?.dimmed) && "opacity-35",
      )}
    >
      <Handle type="target" position={Position.Top} className="!size-1 !min-w-0 !border-0 !bg-transparent" isConnectable={false} />
      <Icon className="size-3.5" aria-hidden />
      {t(closing)}
      <Handle type="source" position={Position.Bottom} className="!size-1 !min-w-0 !border-0 !bg-transparent" isConnectable={false} />
    </div>
  );
}
