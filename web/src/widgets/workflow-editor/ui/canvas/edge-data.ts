import type { Edge } from "@xyflow/react";

import type { EdgeLabel, Target } from "@/entities/workflow";

export type CanvasEdgeData = { label?: EdgeLabel; slot?: Target; taken?: boolean; dragging?: boolean; right?: number; tone?: string; dimmed?: boolean };

export type CanvasEdge = Edge<CanvasEdgeData, "flow" | "again">;
