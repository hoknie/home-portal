import { fireEvent, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import { vi } from "vitest";

import { type Step, type Workflow, portalValuesSchema, workflowAddressOf, workflowCatalogueSchema, workflowsSchema } from "@/entities/workflow";
import { apiSamples } from "@/shared/api";
import { routes } from "@/shared/config";
import { pushAddress, useAddress } from "@/shared/lib/navigation";
import { renderWithProviders } from "@/shared/lib/testing";

import type { Draft } from "../model/draft";
import type { Sources } from "../model/editor-context";
import { WorkflowEditor } from "./workflow-editor";
import { type WorkflowView, WorkflowViewer } from "./workflow-viewer";

export const catalogue = workflowCatalogueSchema.parse(apiSamples.workflowCatalogue);
export const sampleWorkflows = workflowsSchema.parse(apiSamples.workflows).workflows;

class FakeResizeObserver {
  constructor(private readonly callback: ResizeObserverCallback) {}
  observe(target: Element) {
    this.callback([{ target, contentRect: { width: 1200, height: 800 } } as unknown as ResizeObserverEntry], this as unknown as ResizeObserver);
  }
  unobserve() {}
  disconnect() {}
}

class FakeMatrix {
  m22: number;
  constructor(transform?: string) {
    const scale = /scale\(([\d.]+)\)/.exec(transform ?? "")?.[1];
    this.m22 = scale === undefined ? 1 : Number(scale);
  }
}

export function stubCanvasDom() {
  for (const [name, value] of [
    ["ResizeObserver", FakeResizeObserver],
    ["DOMMatrixReadOnly", FakeMatrix],
  ] as const) {
    try {
      Object.assign(globalThis, { [name]: value });
    } catch {
      continue;
    }
  }
  Object.defineProperties(HTMLElement.prototype, {
    offsetHeight: { configurable: true, get(this: HTMLElement) { return Number.parseFloat(this.style.height) || 800; } },
    offsetWidth: { configurable: true, get(this: HTMLElement) { return Number.parseFloat(this.style.width) || 1200; } },
  });
  if (!("getBBox" in SVGElement.prototype)) {
    Object.defineProperty(SVGElement.prototype, "getBBox", { configurable: true, value: () => ({ x: 0, y: 0, width: 0, height: 0 }) });
  }
}

export function withSteps(steps: Step[], extra: Partial<Workflow> = {}): Workflow {
  return { ...sampleWorkflows[1], id: "draft", title: "Draft", used_by: [], steps, ...extra };
}

function sourcesFor(scripts?: Sources["scripts"]): Omit<Sources, "workflows"> {
  return {
        services: [
          { id: "nas", name: "NAS" },
          { id: "router", name: "Router" },
        ],
        states: ["up", "degraded", "down", "unreadable"],
        scripts: scripts ?? [{ path: "restart.sh", runnable: true, problem: null }],
        secrets: [{ name: "token", set: true }],
        channels: [
          { name: "telegram", readiness: "ready" },
          { name: "sms", readiness: "missing" },
        ],
        eventFields: [{ name: "service.id", sample: "nas" }],
        portal: portalValuesSchema.parse(apiSamples.workflowPortal),
        automations: [
          { id: "restart-media", title: "Restart media", event: "service.status-changed", enabled: true },
          { id: "sleeping", title: "Sleeping", event: "portal.started", enabled: false },
        ],
  };
}

export function openEditor(workflow: Workflow | null, options: { initial?: Draft | null; onConflict?: () => void; scripts?: Sources["scripts"]; lastShownRun?: string | null } = {}) {
  const onSaved = vi.fn();
  const onCancel = vi.fn();
  const onConflict = options.onConflict ?? vi.fn();
  const result = renderWithProviders(
    <WorkflowEditor
      workflow={workflow}
      initial={options.initial ?? null}
      revision='"r1"'
      workflows={sampleWorkflows}
      catalogue={catalogue}
      sources={sourcesFor(options.scripts)}
      tags={[]}
      lastShownRun={options.lastShownRun ?? null}
      onSaved={onSaved}
      onCancel={onCancel}
      onConflict={onConflict}
    />,
  );
  return { onSaved, onCancel, onConflict, client: result.client };
}

export function openPage(workflow: Workflow, view: WorkflowView = "view", run: string | null = null) {
  const onRunShown = vi.fn();
  const result = renderWithProviders(
    <WorkflowViewer workflow={workflow} view={view} run={run} revision='"r1"' workflows={sampleWorkflows} catalogue={catalogue} sources={sourcesFor()} tags={[]} onRunShown={onRunShown} />,
  );
  return { onRunShown, ...result };
}

export async function node(name: string) {
  return screen.findByRole("group", { name });
}

export async function addFromSlot(slot: HTMLElement, kind: string) {
  fireEvent.click(slot);
  const dialog = await screen.findByRole("dialog");
  await userEvent.click(within(dialog).getByRole("button", { name: new RegExp(`^${kind}`) }));
}

export function inspector() {
  return screen.getByRole("complementary");
}

export function sentBody(fetch: ReturnType<typeof vi.fn>) {
  const call = fetch.mock.calls.find(([, init]) => ["POST", "PUT"].includes(String((init as RequestInit | undefined)?.method)) && String((init as RequestInit).body).includes("steps"));
  return JSON.parse(String((call?.[1] as RequestInit).body));
}

export function pressOnCanvas(element: HTMLElement) {
  fireEvent.click(element);
}

function AddressHarness({ workflow, start }: { workflow: Workflow; start: string }) {
  const { path } = useAddress();
  const [shown, setShown] = useState<string | null>(null);
  const address = workflowAddressOf(path ?? start);
  if (address.kind === "edit") {
    return (
      <WorkflowEditor
        key="edit"
        workflow={workflow}
        revision='"r1"'
        workflows={sampleWorkflows}
        catalogue={catalogue}
        sources={sourcesFor()}
        tags={[]}
        lastShownRun={shown}
        onSaved={(id) => pushAddress(routes.workflow(id))}
        onCancel={() => pushAddress(routes.workflow(workflow.id))}
        onConflict={vi.fn()}
      />
    );
  }
  const view: WorkflowView = address.kind === "run" ? "run" : address.kind === "history" ? "history" : "view";
  return (
    <WorkflowViewer
      key="page"
      workflow={workflow}
      view={view}
      run={address.kind === "run" ? address.run : null}
      revision='"r1"'
      workflows={sampleWorkflows}
      catalogue={catalogue}
      sources={sourcesFor()}
      tags={[]}
      onRunShown={(_workflow, run) => setShown(run)}
    />
  );
}

export function openAt(workflow: Workflow, path: string) {
  window.history.replaceState(null, "", path);
  return renderWithProviders(<AddressHarness workflow={workflow} start={path} />);
}
