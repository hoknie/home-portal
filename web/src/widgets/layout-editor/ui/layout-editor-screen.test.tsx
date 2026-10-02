import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, expect, it, vi } from "vitest";

import { jsonResponse } from "@/shared/lib/testing";

import { COLUMN_PIXELS, ROW_STEP, mockGeometry, serve } from "./testing";

vi.mock("next/navigation", () => ({
  usePathname: () => "/admin/layout/",
  useRouter: () => ({ replace: vi.fn(), push: vi.fn() }),
  useSearchParams: () => new URLSearchParams(),
}));

type Placement = { key: string | null; widget: string; section: string; column: number | null; row: number | null; width: number; height: unknown };

beforeEach(mockGeometry);

afterEach(() => {
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

const tick = () => new Promise((resolve) => setTimeout(resolve, 10));

async function press(target: Element | Document, key: string, code = key) {
  await act(async () => {
    fireEvent.keyDown(target, { key, code });
    await tick();
  });
}

const tileOf = (key: string) => document.querySelector(`[data-widget="${key}"]`) as HTMLElement;

const order = (section: string) => [...document.querySelectorAll(`[data-section="${section}"] [data-widget]`)].map((element) => element.getAttribute("data-widget"));

async function saved(sent: ReturnType<typeof serve>) {
  await userEvent.click(screen.getByRole("button", { name: "Save layout" }));
  await waitFor(() => expect(sent.some((request) => request.path === "/api/dashboard")).toBe(true));
  const put = sent.find((request) => request.path === "/api/dashboard");
  return { revision: put?.revision, body: put?.body as { sections: Array<{ id: string }>; widgets: Placement[] } };
}

it("moves a widget by keyboard on its left-edge handle: sideways by a column, up and down through the order, and saves it", async () => {
  const sent = serve();
  const handle = await screen.findByRole("button", { name: "Move “Summary”" });
  expect(tileOf("#0").querySelector("[data-tile-content]")?.contains(handle)).toBe(false);
  expect(order("now")).toEqual(["#1", "#0"]);
  await press(handle, "ArrowRight");
  await press(handle, "ArrowRight");
  await press(handle, "ArrowUp");
  expect(order("now")).toEqual(["#0", "#1"]);
  expect(tileOf("#0")).toHaveAttribute("data-column", "3");
  expect(document.querySelector("[data-resize-notice]")).toHaveTextContent("“Summary” is at column 3, place 1 of 2 in “Now”.");
  const { revision, body } = await saved(sent);
  expect(revision).toBe('"r1"');
  expect(body.widgets[0]).toEqual({ key: "#0", widget: "status-summary", section: "now", column: 3, row: 1, width: 8, height: "auto" });
  expect(body.widgets[1]).toMatchObject({ key: "#1", widget: "riga", column: 9, row: 2 });
});

it("drags a widget with a live placeholder, and it lands where the placeholder stood", async () => {
  const sent = serve();
  const width = await screen.findByRole("slider", { name: "Width of the widget “Summary”" });
  for (let step = 0; step < 4; step += 1) {
    fireEvent.keyDown(width, { key: "ArrowLeft" });
  }
  const handle = within(tileOf("#1")).getByRole("button", { name: "Move “Weather”" });
  fireEvent.pointerDown(handle, { pointerId: 1, button: 0, clientX: 8 * COLUMN_PIXELS + 5, clientY: 5 });
  fireEvent.pointerMove(window, { pointerId: 1, clientX: 8 * COLUMN_PIXELS + 10, clientY: 5 * ROW_STEP });
  expect(tileOf("#1")).toHaveAttribute("data-placeholder", "true");
  expect(order("now")).toEqual(["#0", "#1"]);
  fireEvent.pointerUp(window, { pointerId: 1 });
  expect(tileOf("#1")).not.toHaveAttribute("data-placeholder");
  const { body } = await saved(sent);
  expect(body.widgets.slice(0, 2)).toEqual([
    { key: "#0", widget: "status-summary", section: "now", column: 1, row: 1, width: 4, height: "auto" },
    { key: "#1", widget: "riga", section: "now", column: 9, row: 2, width: 4, height: "auto" },
  ]);
});

it("a drag over another widget moves it aside before the release, and Escape puts everything back", async () => {
  serve();
  const handle = await screen.findByRole("button", { name: "Move “Summary”" });
  fireEvent.pointerDown(handle, { pointerId: 1, button: 0, clientX: 5, clientY: 3 * ROW_STEP + 5 });
  fireEvent.pointerMove(window, { pointerId: 1, clientX: 5, clientY: 5 });
  expect(order("now")).toEqual(["#0", "#1"]);
  fireEvent.keyDown(window, { key: "Escape" });
  fireEvent.pointerUp(window, { pointerId: 1 });
  expect(order("now")).toEqual(["#1", "#0"]);
  expect(screen.queryByText("There are unsaved changes")).not.toBeInTheDocument();
  const weather = within(tileOf("#1")).getByRole("button", { name: "Move “Weather”" });
  for (let step = 0; step < 8; step += 1) {
    await press(weather, "ArrowLeft");
  }
  expect(tileOf("#1")).toHaveAttribute("data-column", "1");
  expect(Number(tileOf("#0").dataset.row)).toBeGreaterThan(Number(tileOf("#1").dataset.row));
});

it("Page Down moves a widget to the end of the next section", async () => {
  const sent = serve();
  const handle = await screen.findByRole("button", { name: "Move “Weather”" });
  await press(handle, "PageDown");
  expect(order("media")).toEqual(["#2", "#1"]);
  const { body } = await saved(sent);
  expect(body.widgets.find((widget) => widget.key === "#1")).toMatchObject({ section: "media", row: 2 });
  expect(body.widgets.find((widget) => widget.key === "#2")).toMatchObject({ section: "media", row: 1, column: 1 });
});

it("offers no list for a widget's place, size or section", async () => {
  serve();
  await screen.findByRole("button", { name: "Move “Summary”" });
  expect(tileOf("#0").querySelector("select")).toBeNull();
  expect(within(tileOf("#0")).queryByRole("combobox")).not.toBeInTheDocument();
  expect(screen.queryByRole("button", { name: /Move the section up|Move the section down/ })).not.toBeInTheDocument();
  expect(screen.queryByText("Preview")).not.toBeInTheDocument();
});

it("resizes a widget with the arrows on its edge and announces the new width", async () => {
  const sent = serve();
  const edge = await screen.findByRole("slider", { name: "Width of the widget “Weather”" });
  expect(edge).toHaveAttribute("aria-valuetext", "4 of 12 columns");
  fireEvent.keyDown(edge, { key: "ArrowLeft" });
  expect(tileOf("#1")).toHaveAttribute("data-width", "3");
  expect(document.querySelector("[data-resize-notice]")).toHaveTextContent("Width of the widget “Weather”: 3 of 12 columns.");
  const { body } = await saved(sent);
  expect(body.widgets.find((widget) => widget.key === "#1")).toMatchObject({ width: 3, height: "auto" });
});

it("resizes by dragging the edge and the corner, and Escape keeps the size", async () => {
  serve();
  const edge = await screen.findByRole("slider", { name: "Width of the widget “Summary”" });
  fireEvent.pointerDown(edge, { pointerId: 1, clientX: 800 });
  fireEvent.pointerMove(edge, { pointerId: 1, clientX: 800 - 2 * COLUMN_PIXELS });
  fireEvent.keyDown(window, { key: "Escape" });
  fireEvent.pointerUp(edge, { pointerId: 1 });
  expect(tileOf("#0")).toHaveAttribute("data-width", "8");
  const corner = screen.getByRole("slider", { name: "Size of the widget “Summary”" });
  fireEvent.pointerDown(corner, { pointerId: 2, clientX: 800, clientY: 100 });
  fireEvent.pointerMove(corner, { pointerId: 2, clientX: 800 - 2 * COLUMN_PIXELS, clientY: 100 + ROW_STEP });
  expect(within(tileOf("#0")).getByText("6 × 3")).toBeInTheDocument();
  fireEvent.pointerUp(corner, { pointerId: 2 });
  expect(tileOf("#0")).toHaveAttribute("data-width", "6");
  expect(tileOf("#0")).toHaveAttribute("data-height", "3");
});

it("steps the height by keyboard and goes back to the content's height above one row", async () => {
  serve();
  const bottom = await screen.findByRole("slider", { name: "Height of the widget “Weather”" });
  fireEvent.keyDown(bottom, { key: "ArrowDown" });
  expect(tileOf("#1")).toHaveAttribute("data-height", "1");
  fireEvent.keyDown(bottom, { key: "ArrowUp" });
  expect(tileOf("#1")).toHaveAttribute("data-height", "auto");
});

it("reorders sections by dragging their handle with the keyboard and saves them in the new order", async () => {
  const sent = serve();
  const handle = await screen.findByRole("button", { name: "Move the section “Now”" });
  handle.focus();
  await press(handle, " ", "Space");
  await press(document, "ArrowDown");
  await press(document, " ", "Space");
  await waitFor(() =>
    expect([...document.querySelectorAll("section[data-section-block]")].map((section) => section.getAttribute("data-section"))).toEqual(["media", "now"]),
  );
  const { body } = await saved(sent);
  expect(body.sections.map((section) => section.id)).toEqual(["media", "now"]);
  expect(body.widgets.map((widget) => widget.key)).toEqual(["#2", "#0", "#1"]);
});

it("undoes a resize with the keyboard and redoes it, and undoing to the loaded layout leaves no changes", async () => {
  serve();
  const edge = await screen.findByRole("slider", { name: "Width of the widget “Summary”" });
  fireEvent.keyDown(edge, { key: "ArrowLeft" });
  fireEvent.keyDown(document.body, { key: "z", ctrlKey: true });
  expect(tileOf("#0")).toHaveAttribute("data-width", "8");
  expect(document.querySelector("[data-resize-notice]")).toHaveTextContent("Undone: resizing “Summary”.");
  expect(screen.queryByText("There are unsaved changes")).not.toBeInTheDocument();
  fireEvent.keyDown(document.body, { key: "z", metaKey: true, shiftKey: true });
  expect(tileOf("#0")).toHaveAttribute("data-width", "7");
});

it("asks before leaving with unsaved changes", async () => {
  serve();
  await screen.findByRole("button", { name: "Move “Summary”" });
  const confirm = vi.spyOn(window, "confirm").mockReturnValue(false);
  const link = document.createElement("a");
  link.href = "/";
  link.textContent = "home";
  document.body.append(link);
  fireEvent.keyDown(within(tileOf("#1")).getByRole("slider", { name: /^Width/ }), { key: "ArrowLeft" });
  await userEvent.click(link);
  expect(confirm).toHaveBeenCalledWith("Leave the page? Unsaved layout changes will be lost.");
  link.remove();
});

it("on a conflict it keeps the arrangement, and a server error lands on the widget it names", async () => {
  serve({ layoutAnswer: () => new Response("stale", { status: 409 }) });
  await screen.findByRole("button", { name: "Move “Summary”" });
  fireEvent.keyDown(within(tileOf("#1")).getByRole("slider", { name: /^Width/ }), { key: "ArrowLeft" });
  await userEvent.click(screen.getByRole("button", { name: "Save layout" }));
  expect(await screen.findByRole("button", { name: "Overwrite" })).toBeInTheDocument();
  expect(tileOf("#1")).toHaveAttribute("data-width", "3");
});

it("shows a server error on the widget it names", async () => {
  serve({ layoutAnswer: () => jsonResponse({ errors: [{ field: "widgets[1].column", message: "must leave room for the width" }] }, { status: 422 }) });
  await screen.findByRole("button", { name: "Move “Summary”" });
  fireEvent.keyDown(within(tileOf("#1")).getByRole("slider", { name: /^Width/ }), { key: "ArrowLeft" });
  await userEvent.click(screen.getByRole("button", { name: "Save layout" }));
  expect(await within(tileOf("#1")).findByText("widgets[1].column: must leave room for the width")).toBeInTheDocument();
});

it("places a widget chosen from the library, and removing a place leaves the library alone", async () => {
  const sent = serve();
  await screen.findByRole("button", { name: "Move “Summary”" });
  const media = document.querySelector('[data-section-block][data-section="media"]') as HTMLElement;
  await userEvent.click(within(media).getByRole("button", { name: "Add widget" }));
  const picker = screen.getByRole("dialog", { name: "Place a widget" });
  expect(within(picker).getByRole("link", { name: "Open the widget library" })).toHaveAttribute("href", expect.stringMatching(/^\/admin\/layout\/widgets\/?$/));
  await userEvent.type(within(picker).getByRole("searchbox", { name: "Search" }), "disk");
  expect(within(picker).queryByText("Riga")).not.toBeInTheDocument();
  await userEvent.click(within(picker).getByRole("button", { name: /^Disks/ }));
  await userEvent.click(within(tileOf("#1")).getByRole("button", { name: "Remove" }));
  const asking = screen.getByRole("dialog", { name: /Remove/ });
  expect(asking).toHaveTextContent("The widget leaves this page and stays in the library.");
  await userEvent.click(within(asking).getByRole("button", { name: "Remove" }));
  const { body } = await saved(sent);
  expect(body.widgets.map((widget) => widget.widget)).toEqual(["status-summary", "roads", "disks"]);
  expect(body.widgets[2]).toEqual({ key: null, widget: "disks", section: "media", column: null, row: null, width: 4, height: 2 });
  expect(sent.some((request) => request.path.startsWith("/api/dashboard/library"))).toBe(false);
});

it("draws each widget's real content, which takes no input, and an unknown type as a placeholder", async () => {
  serve();
  await screen.findByRole("button", { name: "Move “Summary”" });
  const content = tileOf("#0").querySelector("[data-tile-content]") as HTMLElement;
  expect(content).toHaveAttribute("inert");
  expect(within(content).getByText("Services in total")).toBeInTheDocument();
  expect(within(tileOf("#2")).getByText(/traffic/)).toBeInTheDocument();
  expect(within(tileOf("#0")).queryByRole("button", { name: "Duplicate" })).not.toBeInTheDocument();
});
