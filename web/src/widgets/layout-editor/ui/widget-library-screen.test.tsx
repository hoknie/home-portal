import { screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, expect, it, vi } from "vitest";

import { DEFAULT_APPEARANCE, apiSamples } from "@/shared/api";
import { jsonResponse, renderWithProviders } from "@/shared/lib/testing";

import { WidgetLibraryScreen } from "./widget-library-screen";

const navigation = vi.hoisted(() => ({ replace: vi.fn(), push: vi.fn() }));
let search = "";

vi.mock("next/navigation", () => ({
  usePathname: () => "/admin/layout/widgets/",
  useRouter: () => ({ replace: navigation.replace, push: navigation.push }),
  useSearchParams: () => new URLSearchParams(search),
}));

const entry = { title: null, environments: null, public: false, appearance: DEFAULT_APPEARANCE, width: 12, height: "auto" };

const LIBRARY = {
  widgets: [
    { ...entry, id: "riga", type: "weather", settings: { latitude: 56.95, longitude: 24.11 }, placed: 1 },
    { ...entry, id: "disks", type: "custom", title: "Disks", placed: 0, settings: { source: { workflow: "revive" }, blocks: [{ kind: "stat", label: "Free", value: "{{data.free}}" }] } },
  ],
};

type Sent = { path: string; method: string; body: unknown; revision: string | null };

function serve({ automationsOn = true, previewErrors = [] as { field: string; message: string }[] } = {}) {
  const sent: Sent[] = [];
  const fetch = vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
    const path = String(input);
    const method = init?.method ?? "GET";
    const request = { path, method, body: init?.body ? JSON.parse(String(init.body)) : null, revision: new Headers(init?.headers).get("If-Match") };
    if (method !== "GET") {
      sent.push(request);
    }
    if (path === "/api/widgets/preview") {
      return jsonResponse({ blocks: [], errors: previewErrors, data: { free: 120, total: 500 }, ran: true, problem: null, paths: [] });
    }
    if (path.startsWith("/api/dashboard/library") && method === "DELETE") {
      return new Response(null, { status: 204, headers: { ETag: '"r2"' } });
    }
    if (path.startsWith("/api/dashboard/library") && method !== "GET") {
      return jsonResponse({ ...(request.body as object), placed: 0 }, { status: method === "POST" ? 201 : 200, headers: { ETag: '"r2"' } });
    }
    if (path === "/api/dashboard/library") {
      return jsonResponse(LIBRARY, { headers: { ETag: '"r1"' } });
    }
    if (path === "/api/environment") {
      return jsonResponse({ environment: "local", detected: "local", switchable: true, environments: ["local", "internet"] });
    }
    if (path === "/api/modules") {
      const modules = structuredClone(apiSamples.modules) as { modules: Array<{ name: string; enabled: boolean }> };
      for (const switched of modules.modules) {
        switched.enabled = switched.name !== "automations" || automationsOn;
      }
      return jsonResponse(modules, { headers: { ETag: '"m"' } });
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
  renderWithProviders(<WidgetLibraryScreen />);
  return sent;
}

const cardOf = (id: string) => document.querySelector(`[data-library-widget="${id}"]`) as HTMLElement;

beforeEach(() => {
  search = "";
});

afterEach(() => {
  navigation.replace.mockReset();
  navigation.push.mockReset();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

it("lists the library as cards with their content and use, and only an unplaced widget can be deleted", async () => {
  const sent = serve();
  await waitFor(() => expect(cardOf("disks")).not.toBeNull());
  expect(cardOf("riga")).toHaveTextContent("1 placement");
  expect(cardOf("disks")).toHaveTextContent("0 placements");
  expect(cardOf("riga").querySelector("[inert]")).not.toBeNull();
  const placedDelete = within(cardOf("riga")).getByRole("button", { name: /^Delete/ });
  expect(placedDelete).toBeDisabled();
  expect(placedDelete).toHaveAttribute("title", "Remove it from the layout before deleting it");
  await userEvent.click(within(cardOf("disks")).getByRole("button", { name: /^Delete/ }));
  await userEvent.click(within(screen.getByRole("dialog", { name: "Delete “Disks”?" })).getByRole("button", { name: "Delete" }));
  await waitFor(() => expect(sent).toEqual([expect.objectContaining({ path: "/api/dashboard/library/disks", method: "DELETE", revision: '"r1"' })]));
});

it("duplicates a widget as a new entry with a numbered id", async () => {
  const sent = serve();
  await waitFor(() => expect(cardOf("disks")).not.toBeNull());
  await userEvent.click(within(cardOf("disks")).getByRole("button", { name: /^Duplicate/ }));
  await waitFor(() => expect(sent).toHaveLength(1));
  expect(sent[0]).toMatchObject({ path: "/api/dashboard/library", method: "POST", revision: '"r1"', body: { id: "disks-2", type: "custom", title: "Disks" } });
});

it("a custom widget from the gallery or a template opens the widget builder", async () => {
  serve();
  await waitFor(() => expect(cardOf("disks")).not.toBeNull());
  await userEvent.click(screen.getAllByRole("button", { name: "Add widget" })[0]);
  const gallery = screen.getByRole("dialog", { name: "Add a widget" });
  await userEvent.type(within(gallery).getByRole("searchbox", { name: "Search" }), "list");
  await userEvent.click(within(gallery).getByRole("button", { name: /^A list/ }));
  expect(navigation.push).toHaveBeenCalledWith("/admin/layout/widgets/new/?template=list&back=library");
  await userEvent.click(within(cardOf("disks")).getByRole("button", { name: /^Settings/ }));
  expect(navigation.push).toHaveBeenCalledWith("/admin/layout/widgets/edit/?id=disks&back=library");
});

it("the gallery shows a custom widget disabled while the automations module is off", async () => {
  serve({ automationsOn: false });
  await waitFor(() => expect(cardOf("disks")).not.toBeNull());
  await userEvent.click(screen.getAllByRole("button", { name: "Add widget" })[0]);
  const gallery = screen.getByRole("dialog", { name: "Add a widget" });
  await waitFor(() => expect(within(gallery).getByRole("button", { name: /^Custom widget/ })).toBeDisabled());
  expect(within(gallery).getByRole("button", { name: /^Custom widget/ })).toHaveTextContent("The automations module is off");
  expect(within(gallery).getByRole("button", { name: /^Weather/ })).toBeEnabled();
});

it("an address naming a widget opens its settings, and closing them drops the parameter", async () => {
  search = "widget=riga";
  serve();
  const dialog = await screen.findByRole("dialog", { name: "Weather" });
  expect(within(dialog).queryByRole("tab", { name: "Data" })).toBeNull();
  await userEvent.click(within(dialog).getByRole("button", { name: "Cancel" }));
  expect(navigation.replace).toHaveBeenCalledWith("/admin/layout/widgets/", { scroll: false });
});

it("an address naming a custom widget goes to its builder", async () => {
  search = "widget=disks";
  serve();
  await waitFor(() => expect(navigation.replace).toHaveBeenCalledWith("/admin/layout/widgets/edit/?id=disks&back=library", { scroll: false }));
});
