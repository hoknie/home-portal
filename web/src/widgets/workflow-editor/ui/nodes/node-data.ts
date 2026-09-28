import type { Node } from "@xyflow/react";

import type { FlowNode, FlowNodeType } from "@/entities/workflow";

export type NodePath = { order?: number; dimmed: boolean; reached: boolean };

export type CanvasNode = Node<{ node: FlowNode; path: NodePath | null }, FlowNodeType>;
