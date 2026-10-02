import { vi } from "vitest";

import { DEFAULT_APPEARANCE, apiSamples } from "@/shared/api";
import { jsonResponse, renderWithProviders } from "@/shared/lib/testing";

import { WidgetBuilderScreen } from "./widget-builder-screen";

type Raw = Record<string, unknown> & { kind: string };

const LEAVES: Record<string, (raw: Raw) => Record<string, unknown>> = {
  stat: (raw) => ({ label: raw.label ?? "", value: raw.value ?? "", unit: null, caption: null, icon: null, tone: "neutral" }),
  list: () => ({ items: [], empty: null, more: 0 }),
  progress: (raw) => ({ label: raw.label ?? null, value: 0, maximum: 100, caption: null, tone: "neutral" }),
};

export function rendered(raw: Raw): Record<string, unknown> {
  if (raw.kind === "row" || raw.kind === "column") {
    const blocks = ((raw.blocks as Raw[]) ?? []).map(rendered);
    return raw.kind === "row" ? { kind: "row", align: "stretch", gap: "normal", widths: null, blocks } : { kind: "column", align: null, valign: null, gap: "normal", blocks };
  }
  const leaf = LEAVES[raw.kind];
  return leaf ? { kind: raw.kind, align: null, valign: null, ...leaf(raw) } : { kind: "text", align: null, valign: null, text: String(raw.text ?? ""), size: "normal", weight: "normal", muted: false, tone: "neutral" };
}

export const DISKS = {
  id: "disks",
  type: "custom",
  title: "Disks",
  environments: null,
  public: false,
  appearance: DEFAULT_APPEARANCE,
  width: 12,
  height: "auto",
  placed: 1,
  settings: { source: { workflow: "weather" }, blocks: [{ kind: "stat", label: "Free", value: "{{data.free}}" }] },
};

export type Sent = { path: string; method: string; body: Record<string, unknown> | null; revision: string | null };

export type BuilderServing = {
  blocks?: Raw[];
  saveAnswer?: (sent: Sent) => Response;
  paths?: { path: string; description: string | null; kind: string }[];
  data?: unknown;
  previewErrors?: { field: string; message: string }[];
  layout?: { sections: unknown[]; widgets: unknown[] };
};

export function serveBuilder(mode: "new" | "edit", { blocks, saveAnswer, paths = [], data = null, previewErrors = [], layout = { sections: [], widgets: [] } }: BuilderServing = {}) {
  const sent: Sent[] = [];
  const widget = blocks ? { ...DISKS, settings: { ...DISKS.settings, blocks } } : DISKS;
  const fetch = vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
    const path = String(input);
    const method = init?.method ?? "GET";
    const body = init?.body ? (JSON.parse(String(init.body)) as Record<string, unknown>) : null;
    if (method !== "GET") {
      sent.push({ path, method, body, revision: new Headers(init?.headers).get("If-Match") });
    }
    if (path === "/api/widgets/preview") {
      const settings = (body?.settings ?? {}) as { blocks?: Raw[] };
      return jsonResponse({ blocks: (settings.blocks ?? []).map(rendered), errors: previewErrors, data: body?.run ? { free: 120, total: 500 } : (body?.sample ?? data), ran: Boolean(body?.run), problem: null, paths });
    }
    if (path.startsWith("/api/dashboard/library") && method !== "GET") {
      const answer = saveAnswer?.(sent[sent.length - 1]);
      return answer ?? jsonResponse({ ...body, id: body?.id ?? "custom", placed: 0 }, { status: method === "POST" ? 201 : 200, headers: { ETag: '"r2"' } });
    }
    if (path === "/api/dashboard/library") {
      return jsonResponse({ widgets: [widget] }, { headers: { ETag: '"r1"' } });
    }
    if (path === "/api/dashboard?all=true") {
      return jsonResponse(layout, { headers: { ETag: '"r1"' } });
    }
    if (path === "/api/dashboard" && method === "PUT") {
      return jsonResponse({ ...layout, widgets: (body?.widgets as unknown[]) ?? [] }, { headers: { ETag: '"r3"' } });
    }
    if (path === "/api/environment") {
      return jsonResponse({ environment: "local", detected: "local", switchable: true, environments: ["local", "internet"] });
    }
    if (path === "/api/workflows") {
      return jsonResponse(apiSamples.workflows, { headers: { ETag: '"w"' } });
    }
    if (path === "/api/workflows/catalogue") {
      return jsonResponse(apiSamples.workflowCatalogue);
    }
    if (path === "/api/automations/scripts") {
      return jsonResponse(apiSamples.automationScripts);
    }
    if (path === "/api/services") {
      return jsonResponse(apiSamples.services, { headers: { ETag: '"s"' } });
    }
    return jsonResponse(apiSamples.widgetWeather);
  });
  vi.stubGlobal("fetch", fetch);
  renderWithProviders(<WidgetBuilderScreen mode={mode} />);
  return sent;
}

export function outlineRow(path: string) {
  return document.querySelector(`[data-outline-path="${path}"]`) as HTMLElement;
}

export function pointAt(element: Element | null, rect = { left: 0, top: 0, width: 200, height: 40 }) {
  if (typeof document.elementFromPoint !== "function") {
    document.elementFromPoint = () => null;
  }
  vi.spyOn(document, "elementFromPoint").mockReturnValue(element);
  if (element) {
    vi.spyOn(element, "getBoundingClientRect").mockReturnValue({ ...rect, x: rect.left, y: rect.top, right: rect.left + rect.width, bottom: rect.top + rect.height, toJSON: () => ({}) } as DOMRect);
  }
}
