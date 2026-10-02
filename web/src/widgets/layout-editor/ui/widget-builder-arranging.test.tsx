import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, it, vi } from "vitest";

import { jsonResponse } from "@/shared/lib/testing";

import { outlineRow, pointAt, serveBuilder } from "./builder-testing";

const page = vi.hoisted(() => ({ search: "id=disks" }));

vi.mock("next/navigation", () => ({
  usePathname: () => "/admin/layout/widgets/edit/",
  useRouter: () => ({ replace: vi.fn(), push: vi.fn() }),
  useSearchParams: () => new URLSearchParams(page.search),
}));

afterEach(() => {
  window.localStorage.clear();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

const text = (value: string) => ({ kind: "text", text: value });
const NESTED = [text("a"), { kind: "column", blocks: [text("b"), text("c")] }, { kind: "row", blocks: [{ kind: "column", blocks: [{ kind: "row", blocks: [text("x"), text("y")] }] }, text("z")] }];

const notice = () => document.querySelector("[data-builder-notice]");

async function choose(path: string) {
  await userEvent.click(outlineRow(path).querySelector("[data-outline-select]") as HTMLElement);
}

it("moves the selected block by keyboard and announces its place, and undo brings a deleted block back", async () => {
  const sent = serveBuilder("edit", { blocks: NESTED });
  await screen.findByRole("tree", { name: "Tree" });
  await choose("1.1");
  fireEvent.keyDown(outlineRow("1.1").querySelector("[data-outline-select]") as HTMLElement, { key: "ArrowUp", altKey: true });
  expect(outlineRow("1.0")).toHaveTextContent("c");
  expect(notice()).toHaveTextContent("Text moved to 2.1.");
  await userEvent.click(screen.getByRole("button", { name: "Delete" }));
  expect(outlineRow("1.1")).toBeNull();
  fireEvent.keyDown(document.body, { key: "z", ctrlKey: true });
  expect(outlineRow("1.1")).toHaveTextContent("b");
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() => expect(sent.some((request) => request.method === "PUT")).toBe(true));
  const saved = sent.find((request) => request.method === "PUT")?.body as { settings: { blocks: { blocks?: { text: string }[] }[] } };
  expect(saved.settings.blocks[1].blocks?.map((block) => block.text)).toEqual(["c", "b"]);
});

it("a drop that would nest groups too deep shows no marker and changes nothing", async () => {
  serveBuilder("edit", { blocks: NESTED });
  await screen.findByRole("tree", { name: "Tree" });
  pointAt(outlineRow("2.0.0.0"));
  fireEvent.pointerDown(document.querySelector('[data-palette-kind="row"]') as HTMLElement, { button: 0, clientX: 0, clientY: 0 });
  await act(async () => {
    window.dispatchEvent(new MouseEvent("pointermove", { clientX: 10, clientY: 30 }));
  });
  expect(document.querySelector("[data-drop-marker]")).toBeNull();
  await act(async () => {
    window.dispatchEvent(new MouseEvent("pointerup", { clientX: 10, clientY: 30 }));
  });
  expect(notice()).toHaveTextContent("Groups nest at most three deep");
  expect(screen.queryByText("There are unsaved changes")).not.toBeInTheDocument();
});

it("collapses a group in the outline with its chevron", async () => {
  serveBuilder("edit", { blocks: NESTED });
  await screen.findByRole("tree", { name: "Tree" });
  await userEvent.click(screen.getAllByRole("button", { name: "Collapse Column" })[0]);
  expect(outlineRow("1.0")).toBeNull();
  expect(outlineRow("1").closest("[role=treeitem]")).toHaveAttribute("aria-expanded", "false");
  expect(screen.getByRole("button", { name: "Expand Column" })).toBeInTheDocument();
});

it("selecting a block on the canvas selects it in the outline, and its buttons do not run", async () => {
  serveBuilder("edit");
  const drawn = await waitFor(() => {
    const found = document.querySelector('[data-block-path="0"]');
    expect(found).not.toBeNull();
    return found as HTMLElement;
  });
  await userEvent.click(drawn);
  expect(outlineRow("0").closest("[role=treeitem]")).toHaveAttribute("aria-selected", "true");
  expect(drawn).toHaveAttribute("data-selected", "true");
});

it("a server error selects its block and shows on its field", async () => {
  serveBuilder("edit", {
    blocks: [text("ok"), text("{{steps.x}}")],
    saveAnswer: () => jsonResponse({ errors: [{ field: "blocks[1].text", message: "names {{steps.x}}, which a widget does not know" }] }, { status: 422 }),
  });
  await userEvent.type(await screen.findByRole("textbox", { name: "Widget title" }), "x");
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  expect(await screen.findByText("names {{steps.x}}, which a widget does not know")).toBeInTheDocument();
  expect(outlineRow("1").closest("[role=treeitem]")).toHaveAttribute("aria-selected", "true");
});

it("suggests declared outputs before a run and inserts a path from the data tree", async () => {
  serveBuilder("edit", { paths: [{ path: "data.temperature", description: "°C now", kind: "value" }] });
  await screen.findByRole("tree", { name: "Tree" });
  await choose("0");
  const tree = await screen.findByRole("region", { name: "Data" });
  await userEvent.click(await within(tree).findByRole("button", { name: "Insert data.temperature" }));
  expect(screen.getByLabelText("Value")).toHaveValue("{{data.free}}{{data.temperature}}");
  expect(within(tree).getByText("°C now")).toBeInTheDocument();
});

it("running the source lists its data and a block's field suggests its paths", async () => {
  const sent = serveBuilder("edit");
  await screen.findByRole("tree", { name: "Tree" });
  await choose("0");
  const tree = screen.getByRole("region", { name: "Data" });
  await userEvent.click(within(tree).getByRole("button", { name: "Run now" }));
  await waitFor(() => expect(sent.some((request) => request.path === "/api/widgets/preview" && request.body?.run === true)).toBe(true));
  expect(await within(tree).findByRole("button", { name: "Insert data.total" })).toBeInTheDocument();
  const value = screen.getByLabelText("Value");
  await userEvent.clear(value);
  await userEvent.type(value, "{{{{data.");
  expect(await screen.findByRole("option", { name: /data\.total/ })).toBeInTheDocument();
});

it("an error the preview finds in a block marks the Block tab and the block", async () => {
  serveBuilder("edit", { previewErrors: [{ field: "blocks[0].value", message: "names {{steps.x}}, which a widget does not know" }] });
  expect(await screen.findByRole("tab", { name: "Block, has problems" })).toBeInTheDocument();
  await waitFor(() => expect(outlineRow("0")).toHaveClass("ring-status-degraded/60"));
});

it("a block in a row is placed lower by its vertical position", async () => {
  const sent = serveBuilder("edit", { blocks: [{ kind: "row", blocks: [text("a"), text("b")] }] });
  await screen.findByRole("tree", { name: "Tree" });
  await choose("0.1");
  const group = screen.getByRole("radiogroup", { name: "Vertical position in the row" });
  await userEvent.click(within(group).getByRole("radio", { name: "Bottom" }));
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() => expect(sent.some((request) => request.method === "PUT")).toBe(true));
  const saved = sent.find((request) => request.method === "PUT")?.body as { settings: { blocks: { blocks: Record<string, unknown>[] }[] } };
  expect(saved.settings.blocks[0].blocks[1]).toEqual({ kind: "text", text: "b", valign: "end" });
});
