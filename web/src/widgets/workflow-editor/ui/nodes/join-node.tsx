"use client";

import { Handle, Position } from "@xyflow/react";

export function JoinNode() {
  return (
    <div className="size-full rounded-full bg-muted-foreground/60" aria-hidden>
      <Handle type="target" position={Position.Top} className="!size-1 !min-w-0 !border-0 !bg-transparent" isConnectable={false} />
      <Handle type="source" position={Position.Bottom} className="!size-1 !min-w-0 !border-0 !bg-transparent" isConnectable={false} />
      <Handle id="again" type="source" position={Position.Right} className="!size-1 !min-w-0 !border-0 !bg-transparent" isConnectable={false} />
    </div>
  );
}
