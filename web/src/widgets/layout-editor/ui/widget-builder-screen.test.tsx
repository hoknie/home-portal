import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, it, vi } from "vitest";

import { DISKS, outlineRow, serveBuilder } from "./builder-testing";

const page = vi.hoisted(() => ({ search: "", replace: vi.fn(), push: vi.fn() }));

vi.mock("next/navigation", () => ({
  usePathname: () => "/admin/layout/widgets/edit/",
  useRouter: () => ({ replace: page.replace, push: page.push }),
  useSearchParams: () => new URLSearchParams(page.search),
}));

afterEach(() => {
  window.localStorage.clear();
  page.search = "";
  page.replace.mockReset();
  page.push.mockReset();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

it("a new widget from a template is saved to the library and the address moves to its edit form", async () => {
  page.search = "template=list";
  const sent = serveBuilder("new");
  expect(await screen.findByRole("tab", { name: "Data" })).toHaveAttribute("aria-selected", "true");
  await userEvent.click(screen.getByRole("tab", { name: "Block" }));
  expect(within(screen.getByRole("tree", { name: "Tree" })).getByText("List")).toBeInTheDocument();
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  const posted = () => sent.find((request) => request.method === "POST" && request.path === "/api/dashboard/library");
  await waitFor(() => expect(posted()).toBeDefined());
  expect(posted()).toMatchObject({ path: "/api/dashboard/library", revision: '"r1"', body: { id: null, type: "custom" } });
  await waitFor(() => expect(page.replace).toHaveBeenCalledWith("/admin/layout/widgets/edit/?id=custom&back=library", { scroll: false }));
});

it("a conflict says the configuration changed and offers to overwrite", async () => {
  page.search = "id=disks";
  serveBuilder("edit", { saveAnswer: () => new Response("stale", { status: 409 }) });
  const title = await screen.findByRole("textbox", { name: "Widget title" });
  await userEvent.type(title, " now");
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  expect(await screen.findByRole("button", { name: "Overwrite" })).toBeInTheDocument();
  expect(title).toHaveValue("Disks now");
});

it("a row dragged in gets a stat and a column into its slots, and saving writes the nested blocks", async () => {
  page.search = "id=disks";
  const sent = serveBuilder("edit");
  await screen.findByRole("tree", { name: "Tree" });
  const drag = async (kind: string, onto: Element) => {
    const { pointAt } = await import("./builder-testing");
    pointAt(onto);
    const source = document.querySelector(`[data-palette-kind="${kind}"]`) as HTMLElement;
    fireEvent.pointerDown(source, { button: 0, clientX: 0, clientY: 0 });
    await act(async () => {
      window.dispatchEvent(new MouseEvent("pointermove", { clientX: 20, clientY: 30 }));
    });
    expect(document.querySelector("[data-drop-marker]")).not.toBeNull();
    await act(async () => {
      window.dispatchEvent(new MouseEvent("pointerup", { clientX: 20, clientY: 30 }));
    });
    vi.restoreAllMocks();
  };
  await drag("row", document.querySelector('[data-outline-slot=""]') as HTMLElement);
  await drag("stat", outlineRow("1.0"));
  await drag("column", outlineRow("1.1"));
  const outline = screen.getByRole("tree", { name: "Tree" });
  expect(within(outline).getAllByRole("treeitem").map((item) => item.querySelector("[data-outline-select] span")?.textContent)).toEqual(["Number", "Row", "Number", "Column", "Empty place", "Empty place"]);
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  expect(await screen.findByText("Needs Label")).toBeInTheDocument();
  expect(outlineRow("1.0").closest("[role=treeitem]")).toHaveAttribute("aria-selected", "true");
  expect(outlineRow("1.1.0")).toHaveTextContent("Empty place");
  expect(outlineRow("1.1.0")).not.toHaveClass("ring-status-degraded/60");
  expect(sent.filter((request) => request.path.startsWith("/api/dashboard/library"))).toHaveLength(0);
});

it("pressing a palette block while an empty place is selected puts it there, a text stays editable, and saving leaves empty places out", async () => {
  page.search = "id=disks";
  const sent = serveBuilder("edit");
  await screen.findByRole("tree", { name: "Tree" });
  await userEvent.click(document.querySelector('[data-palette-kind="column"]') as HTMLElement);
  await userEvent.click(outlineRow("1.0").querySelector("[data-outline-select]") as HTMLElement);
  expect(screen.getByText(/Drag a block here from the palette/)).toBeInTheDocument();
  await userEvent.click(document.querySelector('[data-palette-kind="text"]') as HTMLElement);
  expect(outlineRow("1.0")).toHaveTextContent("Text");
  await userEvent.type(document.getElementById("builder-1-0-text") as HTMLElement, "Backup ok");
  expect(outlineRow("1.0")).toHaveTextContent("Backup ok");
  expect(outlineRow("1.1")).toHaveTextContent("Empty place");
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() => expect(sent.some((request) => request.method === "PUT")).toBe(true));
  const saved = sent.find((request) => request.method === "PUT")?.body as { settings: { blocks: { blocks?: unknown[] }[] } };
  expect(saved.settings.blocks[1].blocks).toEqual([{ kind: "text", text: "Backup ok" }]);
});

it("the size set over the widget is saved to its one place in the layout", async () => {
  page.search = "id=disks";
  const placed = { ...DISKS, key: "#0", section: "main", column: null, row: null, width: 12, height: "auto" };
  const sent = serveBuilder("edit", { layout: { sections: [{ id: "main", title: null, appearance: { title: "shown", surface: "none" } }], widgets: [placed] } });
  const width = await screen.findByRole("spinbutton", { name: "Width in columns" });
  await waitFor(() => expect(width).toHaveValue(12));
  await userEvent.clear(width);
  await userEvent.type(width, "4");
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() => expect(sent.some((request) => request.path === "/api/dashboard" && request.method === "PUT")).toBe(true));
  const put = sent.find((request) => request.path === "/api/dashboard" && request.method === "PUT");
  expect(put?.revision).toBe('"r1"');
  expect(put?.body).toMatchObject({ widgets: [{ key: "#0", widget: "disks", width: 4 }] });
});

it("hides and shows the side columns", async () => {
  page.search = "id=disks";
  serveBuilder("edit");
  await screen.findByRole("tree", { name: "Tree" });
  await userEvent.click(screen.getByRole("button", { name: "Hide Tree" }));
  expect(screen.queryByRole("tree", { name: "Tree" })).toBeNull();
  await userEvent.click(screen.getByRole("button", { name: "Hide Settings" }));
  expect(screen.queryByRole("tab", { name: "Block" })).toBeNull();
  await userEvent.click(screen.getByRole("button", { name: "Show Tree" }));
  expect(screen.getByRole("tree", { name: "Tree" })).toBeInTheDocument();
  expect(screen.getAllByRole("separator").length).toBeGreaterThan(1);
});

it("access chooses environments as tiles and warns when a public widget leaves the internet out", async () => {
  page.search = "id=disks";
  const sent = serveBuilder("edit");
  await userEvent.click(await screen.findByRole("tab", { name: "Access" }));
  expect(screen.getByRole("checkbox", { name: /Everywhere/ })).toHaveAttribute("aria-checked", "true");
  await userEvent.click(screen.getByRole("switch", { name: "Show without signing in" }));
  await userEvent.click(screen.getByRole("checkbox", { name: /local/ }));
  expect(screen.getByRole("note")).toHaveTextContent("Guests from the internet will not see it");
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() => expect(sent.some((request) => request.method === "PUT")).toBe(true));
  expect(sent.find((request) => request.method === "PUT")?.body).toMatchObject({ environments: ["local"], public: true });
});

it("a widget placed nowhere keeps the size set over it in its library entry", async () => {
  page.search = "template=stat";
  const sent = serveBuilder("new");
  const width = await screen.findByRole("spinbutton", { name: "Width in columns" });
  await userEvent.clear(width);
  await userEvent.type(width, "5");
  await userEvent.type(screen.getByRole("spinbutton", { name: "Height in rows" }), "2");
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  const posted = () => sent.find((request) => request.method === "POST" && request.path === "/api/dashboard/library");
  await waitFor(() => expect(posted()).toBeDefined());
  expect(posted()?.body).toMatchObject({ width: 5, height: 2 });
  expect(sent.some((request) => request.path === "/api/dashboard")).toBe(false);
});
