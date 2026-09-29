"use client";

import "@xyflow/react/dist/style.css";

import {
  Background,
  BackgroundVariant,
  Controls,
  MarkerType,
  MiniMap,
  type NodeChange,
  type NodeTypes,
  type EdgeTypes,
  ReactFlow,
  ReactFlowProvider,
  applyNodeChanges,
  useReactFlow,
} from "@xyflow/react";
import { useTranslations } from "next-intl";
import { useTheme } from "next-themes";
import { useEffect, useMemo, useState } from "react";

import { isActive } from "@/entities/automation";
import { type Flow, type Overlay, type RunPath, flowOf, runPath } from "@/entities/workflow";

import { placeable } from "../../model/checks/placing";
import { DROP_RADIUS, dropTarget, slotPoints } from "../../model/edits/drop";
import { useEditor } from "../../model/editor-context";
import { EmptyNode } from "../nodes/empty-node";
import { EndNode } from "../nodes/end-node";
import { FrameNode } from "../nodes/frame-node";
import { JoinNode } from "../nodes/join-node";
import { MarkerNode } from "../nodes/marker-node";
import type { CanvasNode } from "../nodes/node-data";
import { StartNode } from "../nodes/start-node";
import { StepNode } from "../nodes/step-node";
import type { CanvasEdge } from "./edge-data";
import { FlowEdge, LoopEdge } from "./flow-edge";

const NODE_TYPES: NodeTypes = { start: StartNode, step: StepNode, join: JoinNode, frame: FrameNode, empty: EmptyNode, marker: MarkerNode, end: EndNode };
const EDGE_TYPES: EdgeTypes = { flow: FlowEdge, again: LoopEdge };

function nodesOf(flow: Flow, selected: string | null, fixed: boolean, path: RunPath | null): CanvasNode[] {
  return flow.nodes.map((node) => ({
    id: node.id,
    type: node.type,
    position: { x: node.box.x, y: node.box.y },
    width: node.box.width,
    height: node.box.height,
    data: {
      node,
      path: path ? { order: path.order.get(node.id), reached: path.nodes.has(node.id), dimmed: !path.nodes.has(node.id) && node.type !== "frame" } : null,
    },
    draggable: node.type === "step" && !fixed,
    selectable: node.type === "step" || node.type === "start",
    focusable: node.type === "step" || node.type === "start",
    selected: node.id === selected,
    zIndex: node.type === "frame" ? -1 : 1,
  }));
}

function toneOf(overlay: Overlay, id: string) {
  const run = overlay.get(id);
  if (!run) {
    return "up";
  }
  if (run.running) {
    return "degraded";
  }
  return run.outcome === "failed" || run.outcome === "timed-out" ? "down" : "up";
}

function edgesOf(flow: Flow, taken: (source: string, label?: string) => boolean, dragging: boolean, path: RunPath | null, overlay: Overlay, active: boolean): CanvasEdge[] {
  const frames = new Map(flow.nodes.filter((node) => node.type === "frame").map((node) => [node.id.replace(/:frame$/, ""), node.box.x + node.box.width]));
  return flow.edges.map((edge) => ({
    id: edge.id,
    source: edge.source,
    target: edge.target,
    type: edge.kind === "loop-back" ? "again" : "flow",
    sourceHandle: edge.kind === "loop-back" ? "again" : undefined,
    targetHandle: edge.kind === "loop-back" ? "again" : undefined,
    markerEnd: { type: MarkerType.ArrowClosed, width: 16, height: 16 },
    selectable: false,
    focusable: false,
    animated: Boolean(path?.edges.has(edge.id) && active),
    data: {
      tone: path?.edges.has(edge.id) ? toneOf(overlay, edge.target.replace(/:.*$/, "")) : undefined,
      dimmed: path !== null && !path.edges.has(edge.id),
      label: edge.label,
      slot: edge.slot,
      rail: edge.rail,
      dashed: edge.dashed,
      taken: taken(edge.source, edge.label?.key),
      dragging,
      right: frames.get(edge.target),
    },
  }));
}

function Canvas() {
  const t = useTranslations("workflowEditor.canvas");
  const editor = useEditor();
  const { resolvedTheme } = useTheme();
  const view = useReactFlow();
  const flow = useMemo(() => flowOf(editor.draft.steps), [editor.draft.steps]);
  const points = useMemo(() => slotPoints(flow), [flow]);
  const [dragging, setDragging] = useState(false);
  const run = editor.run;
  const active = run ? isActive(run) : false;
  const path = useMemo(
    () => (run?.trace ? runPath(flow, editor.overlay, run.trace.entries, { active, succeeded: run.outcome.result === "succeeded" }) : null),
    [flow, editor.overlay, run, active],
  );
  const computed = useMemo(() => nodesOf(flow, editor.selected, editor.narrow || editor.readOnly, path), [flow, editor.selected, editor.narrow, editor.readOnly, path]);
  const [nodes, setNodes] = useState<CanvasNode[]>(computed);
  const [shownFrom, setShownFrom] = useState(computed);
  if (shownFrom !== computed) {
    setShownFrom(computed);
    setNodes(computed);
  }
  const revealed = editor.revealed;
  useEffect(() => {
    const box = revealed ? flow.nodes.find((node) => node.id === revealed.id)?.box : undefined;
    if (box) {
      void view.setCenter(box.x + box.width / 2, box.y + box.height / 2, { zoom: 1, duration: 300 });
    }
  }, [revealed, flow, view]);
  useEffect(() => {
    if (editor.narrow) {
      void view.fitView({ padding: 0.1, duration: 200 });
    }
  }, [flow, editor.narrow, view]);
  const overlay = editor.overlay;
  const edges = useMemo(
    () =>
      edgesOf(
        flow,
        (source, label) => {
          const run = overlay.get(source);
          return Boolean(run?.branch && label && run.branch === label);
        },
        dragging,
        path,
        overlay,
        active,
      ),
    [flow, dragging, overlay, path, active],
  );
  return (
    <ReactFlow<CanvasNode, CanvasEdge>
      nodes={nodes}
      edges={edges}
      nodeTypes={NODE_TYPES}
      edgeTypes={EDGE_TYPES}
      colorMode={resolvedTheme === "dark" ? "dark" : "light"}
      fitView
      fitViewOptions={{ padding: 0.15, maxZoom: 1 }}
      minZoom={0.2}
      maxZoom={1.5}
      nodesConnectable={false}
      edgesFocusable={false}
      deleteKeyCode={null}
      selectionKeyCode={null}
      multiSelectionKeyCode={null}
      panOnScroll
      proOptions={{ hideAttribution: true }}
      aria-label={t("label")}
      ariaLabelConfig={{
        "node.a11yDescription.default": t("nodeHelp"),
        "node.a11yDescription.keyboardDisabled": t("nodeHelp"),
        "edge.a11yDescription.default": t("edgeHelp"),
        "controls.ariaLabel": t("controls"),
        "controls.zoomIn.ariaLabel": t("zoomIn"),
        "controls.zoomOut.ariaLabel": t("zoomOut"),
        "controls.fitView.ariaLabel": t("fit"),
        "controls.interactive.ariaLabel": t("controls"),
        "minimap.ariaLabel": t("minimap"),
        "handle.ariaLabel": t("edgeHelp"),
      }}
      onNodesChange={(changes: NodeChange<CanvasNode>[]) => setNodes((current) => applyNodeChanges(changes.filter((change) => change.type !== "remove" && change.type !== "select"), current))}
      onNodeClick={(_, node) => editor.select(node.type === "step" || node.type === "start" ? node.id : null)}
      onPaneClick={() => editor.select(null)}
      onNodeDragStart={() => setDragging(true)}
      onNodeDragStop={(_, node) => {
        setDragging(false);
        const from = node.data.node.path;
        const center = { x: node.position.x + node.data.node.box.width / 2, y: node.position.y + node.data.node.box.height / 2 };
        const moving = node.data.node.step;
        const target = from ? dropTarget(points, from, center, DROP_RADIUS, (candidate) => moving === undefined || placeable(moving, candidate)) : null;
        if (from && target) {
          editor.move(from, target);
        } else {
          setNodes(computed);
        }
      }}
    >
      <Background variant={BackgroundVariant.Dots} gap={20} size={1} />
      <Controls showInteractive={false} position="bottom-left" />
      {editor.narrow ? null : <MiniMap pannable zoomable position="bottom-right" nodeStrokeWidth={2} />}
    </ReactFlow>
  );
}

export default function FlowCanvas() {
  return (
    <ReactFlowProvider>
      <Canvas />
    </ReactFlowProvider>
  );
}
