import { vi } from "vitest";

import { DEFAULT_APPEARANCE, DEFAULT_SECTION_APPEARANCE, apiSamples } from "@/shared/api";
import { jsonResponse, renderWithProviders } from "@/shared/lib/testing";

import { LayoutEditorScreen } from "./layout-editor-screen";

const place = { column: null, row: null, width: 12, height: "auto", appearance: DEFAULT_APPEARANCE, environments: null, public: false, title: null };

export const LAYOUT = {
  sections: [
    { id: "now", title: "Now", appearance: DEFAULT_SECTION_APPEARANCE },
    { id: "media", title: "Media", appearance: DEFAULT_SECTION_APPEARANCE },
  ],
  widgets: [
    { ...place, key: "#0", type: "status-summary", id: "status-summary", settings: {}, section: "now", width: 8 },
    { ...place, key: "#1", type: "weather", id: "riga", settings: { latitude: 56.95, longitude: 24.11 }, section: "now", width: 4, column: 9, row: 1, public: true },
    { ...place, key: "#2", type: "traffic", id: "roads", settings: { city: "Riga", zoom: 12 }, section: "media" },
  ],
};

const entry = { title: null, environments: null, public: false, appearance: DEFAULT_APPEARANCE, width: 12, height: "auto", placed: 1 };

export const LIBRARY = {
  widgets: [
    { ...entry, id: "status-summary", type: "status-summary", settings: {} },
    { ...entry, id: "riga", type: "weather", settings: { latitude: 56.95, longitude: 24.11 }, public: true },
    { ...entry, id: "roads", type: "traffic", settings: { city: "Riga", zoom: 12 } },
    { ...entry, id: "disks", type: "custom", title: "Disks", placed: 0, width: 4, height: 2, settings: { source: { workflow: "revive" }, blocks: [{ kind: "stat", label: "Free", value: "{{data.free}}" }] } },
    { ...entry, id: "home-services", type: "services", title: "Home services", placed: 0, settings: {} },
  ],
};

export type Sent = { path: string; method: string; body: unknown; revision: string | null };

export type Serving = { layoutAnswer?: (sent: Sent) => Response; previewErrors?: { field: string; message: string }[]; layout?: { sections: unknown[]; widgets: unknown[] } };

export function serve({ layoutAnswer = () => jsonResponse(LAYOUT, { headers: { ETag: '"r2"' } }), previewErrors = [], layout = LAYOUT }: Serving = {}) {
  const sent: Sent[] = [];
  const fetch = vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
    const path = String(input);
    const method = init?.method ?? "GET";
    if (init?.body) {
      sent.push({ path, method, body: JSON.parse(String(init.body)), revision: new Headers(init.headers).get("If-Match") });
    }
    if (path === "/api/dashboard" && method === "PUT") {
      return layoutAnswer(sent[sent.length - 1]);
    }
    if (path.startsWith("/api/dashboard/library/") && method === "PUT") {
      const body = sent[sent.length - 1].body as Record<string, unknown>;
      return jsonResponse({ ...body, placed: 1 }, { headers: { ETag: '"r9"' } });
    }
    if (path === "/api/dashboard/library") {
      return jsonResponse(LIBRARY, { headers: { ETag: '"r1"' } });
    }
    if (path === "/api/widgets/preview") {
      return jsonResponse({ blocks: [], errors: previewErrors, data: { free: 120, total: 500 }, ran: true, problem: null, paths: [] });
    }
    if (path === "/api/dashboard?all=true") {
      return jsonResponse(layout, { headers: { ETag: '"r1"' } });
    }
    if (path === "/api/environment") {
      return jsonResponse({ environment: "local", detected: "local", switchable: true, environments: ["local", "vpn", "internet"] });
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
      const services = structuredClone(apiSamples.services) as { services: Array<{ group: string | null }> };
      services.services[1].group = "Network";
      return jsonResponse(services, { headers: { ETag: '"s"' } });
    }
    return jsonResponse(apiSamples.widgetWeather);
  });
  vi.stubGlobal("fetch", fetch);
  renderWithProviders(<LayoutEditorScreen />);
  return sent;
}

export const COLUMN_PIXELS = (1200 - 11 * 16) / 12 + 16;

export const ROW_STEP = 96;

export function mockGeometry() {
  vi.spyOn(Element.prototype, "getBoundingClientRect").mockImplementation(function rect(this: Element) {
    const box = (left: number, top: number, width: number, height: number) =>
      ({ x: left, y: top, left, top, width, height, right: left + width, bottom: top + height, toJSON: () => ({}) }) as DOMRect;
    const blocks = [...document.querySelectorAll("section[data-section-block]")];
    const block = this.closest("section[data-section-block]");
    const top = block ? blocks.indexOf(block) * 2000 : 0;
    if (this.matches("section[data-section-block], [data-section-grid]")) {
      return box(0, top, 1200, 1800);
    }
    const tile = this.closest<HTMLElement>("[data-widget]");
    if (!tile) {
      return box(0, 0, 0, 0);
    }
    const siblings = [...(tile.parentElement?.querySelectorAll(":scope > [data-widget]") ?? [])];
    const column = Number(tile.dataset.column ?? 1);
    const row = Number(tile.dataset.row ?? siblings.indexOf(tile) * 3 + 1);
    const rows = tile.dataset.height === "auto" ? 2 : Number(tile.dataset.height);
    return box((column - 1) * COLUMN_PIXELS, top + (row - 1) * ROW_STEP, Number(tile.dataset.width) * COLUMN_PIXELS - 16, rows * ROW_STEP - 16);
  });
}
