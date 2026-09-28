"use client";

import { Handle, type NodeProps, Position } from "@xyflow/react";
import { Plus } from "lucide-react";
import { useTranslations } from "next-intl";

import { useEditor } from "../../model/editor-context";
import { slotKey } from "../../model/edits/drop";
import type { CanvasNode } from "./node-data";

export function EmptyNode({ id, data }: NodeProps<CanvasNode>) {
  const t = useTranslations("workflowEditor");
  const editor = useEditor();
  const target = data.node.target;
  return (
    <div className="size-full">
      <Handle type="target" position={Position.Top} className="!size-1 !min-w-0 !border-0 !bg-transparent" isConnectable={false} />
      {editor.readOnly ? (
        <div className="size-full rounded-xl border-2 border-dashed border-glass-edge opacity-40" />
      ) : (
        <button
          type="button"
          data-slot={target ? slotKey(target) : id}
          className="nodrag flex size-full items-center justify-center gap-1.5 rounded-xl border-2 border-dashed border-glass-edge text-sm text-muted-foreground transition-colors hover:border-primary hover:text-primary"
          onClick={() => target && editor.openPalette(target)}
        >
          <Plus className="size-4" aria-hidden />
          {t("addStep")}
        </button>
      )}
      <Handle type="source" position={Position.Bottom} className="!size-1 !min-w-0 !border-0 !bg-transparent" isConnectable={false} />
    </div>
  );
}
