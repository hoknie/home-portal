"use client";

import { Handle, Position } from "@xyflow/react";
import { Flag } from "lucide-react";
import { useTranslations } from "next-intl";

export function EndNode() {
  const t = useTranslations("workflowEditor");
  return (
    <div role="group" aria-label={t("endNode")} className="flex size-full items-center justify-center gap-2 rounded-full border border-glass-edge bg-muted text-sm font-medium text-muted-foreground">
      <Handle type="target" position={Position.Top} className="!size-1 !min-w-0 !border-0 !bg-transparent" isConnectable={false} />
      <Flag className="size-3.5" aria-hidden />
      {t("endNode")}
    </div>
  );
}
