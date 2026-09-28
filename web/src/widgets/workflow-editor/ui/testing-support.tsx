import { fireEvent, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { vi } from "vitest";

import { type Step, type Workflow, workflowCatalogueSchema, workflowsSchema } from "@/entities/workflow";
import { apiSamples } from "@/shared/api";
import { renderWithProviders } from "@/shared/lib/testing";

import type { Draft } from "../model/draft";
import { WorkflowEditor } from "./workflow-editor";

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

export function openEditor(workflow: Workflow | null, options: { initial?: Draft | null; onConflict?: () => void } = {}) {
  const onSaved = vi.fn();
  const onConflict = options.onConflict ?? vi.fn();
  const result = renderWithProviders(
    <WorkflowEditor
      workflow={workflow}
      initial={options.initial ?? null}
      revision='"r1"'
      workflows={sampleWorkflows}
      catalogue={catalogue}
      sources={{
        services: [
          { id: "nas", name: "NAS" },
          { id: "router", name: "Router" },
        ],
        states: ["up", "degraded", "down", "unreadable"],
        scripts: [{ path: "restart.sh", runnable: true, problem: null }],
        secrets: [{ name: "token", set: true }],
        channels: [
          { name: "telegram", readiness: "ready" },
          { name: "sms", readiness: "missing" },
        ],
        eventFields: [{ name: "service.id", sample: "nas" }],
        automations: [
          { id: "restart-media", title: "Restart media", event: "service.status-changed", enabled: true },
          { id: "sleeping", title: "Sleeping", event: "portal.started", enabled: false },
        ],
      }}
      tags={[]}
      onSaved={onSaved}
      onConflict={onConflict}
    />,
  );
  return { onSaved, onConflict, client: result.client };
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
