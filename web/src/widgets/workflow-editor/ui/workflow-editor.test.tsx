import { fireEvent, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, expect, it, vi } from "vitest";

import { plainInput } from "@/entities/workflow";
import { jsonResponse } from "@/shared/lib/testing";

import { addFromSlot, inspector, node, openEditor, pressOnCanvas, sampleWorkflows, sentBody, stubCanvasDom, withSteps } from "./testing-support";

vi.mock("next/navigation", () => ({ useRouter: () => ({ push: vi.fn(), replace: vi.fn() }) }));

beforeEach(() => {
  stubCanvasDom();
});

afterEach(() => {
  vi.unstubAllGlobals();
});

async function slot(key: string) {
  return waitFor(() => {
    const found = document.querySelector<HTMLElement>(`[data-slot="${key}"]`);
    expect(found, key).not.toBeNull();
    return found as HTMLElement;
  });
}

it("the canvas draws the start, a node per step and the end", async () => {
  openEditor(withSteps([{ id: "ping", kind: "http", url: "http://nas.lan" }]));
  expect(await node("ping")).toHaveTextContent("GET http://nas.lan");
  expect(await node("Start")).toBeInTheDocument();
  expect(await node("End")).toBeInTheDocument();
});

it("building a retry through the + slots and the inspector saves the same tree", async () => {
  const fetch = vi.fn(async () => jsonResponse(sampleWorkflows[1], { status: 201 }));
  vi.stubGlobal("fetch", fetch);
  const { onSaved } = openEditor(null);
  await addFromSlot(await slot("|steps|0"), "Loop");
  const times = within(inspector()).getByLabelText(/Times/);
  await userEvent.clear(times);
  await userEvent.type(times, "3");
  await addFromSlot(await slot("steps[0]|body|0"), "HTTP request");
  const stepId = within(inspector()).getByLabelText(/Step id/);
  await userEvent.clear(stepId);
  await userEvent.type(stepId, "ping");
  fireEvent.change(within(inspector()).getByRole("combobox", { name: "URL" }), { target: { value: "http://nas.lan/api/ping" } });
  await addFromSlot(await slot("steps[0]|body|1"), "If");
  const left = within(inspector()).getByRole("combobox", { name: "Value" });
  await userEvent.click(left);
  await userEvent.keyboard("{{{{steps.ping.sta");
  await userEvent.keyboard("{Enter}");
  expect(left).toHaveValue("{{steps.ping.status}}");
  await userEvent.type(within(inspector()).getByRole("combobox", { name: "Compared with" }), "200");
  await addFromSlot(await slot("steps[0].body[1]|then|0"), "Stop");
  await addFromSlot(await slot("|steps|1"), "Notification");
  fireEvent.change(within(inspector()).getByRole("combobox", { name: "Text" }), { target: { value: "still down" } });
  pressOnCanvas(await node("Start"));
  await userEvent.type(within(inspector()).getByLabelText("Title"), "Ping NAS");
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() => expect(onSaved).toHaveBeenCalled());
  expect(sentBody(fetch)).toMatchObject({
    id: "ping-nas",
    steps: [
      {
        kind: "loop",
        repeat: 3,
        body: [
          { id: "ping", kind: "http", url: "http://nas.lan/api/ping" },
          { kind: "if", condition: { left: "{{steps.ping.status}}", op: "==", right: "200" }, then: [{ kind: "stop", outcome: "succeeded" }] },
        ],
      },
      { kind: "notify", text: "still down" },
    ],
  });
});

it("moving without dragging puts a node inside a loop from its menu", async () => {
  openEditor(
    withSteps([
      { id: "ping", kind: "http", url: "http://nas.lan" },
      { id: "retry", kind: "loop", repeat: 2, body: [{ id: "pause", kind: "wait", seconds: 1 }] },
    ]),
  );
  const menu = within(await node("ping")).getByRole("button", { name: "Actions for ping" });
  menu.focus();
  await userEvent.keyboard("{Enter}");
  const into = await screen.findByRole("menuitem", { name: "Move into…" });
  into.focus();
  await userEvent.keyboard("{ArrowRight}");
  await userEvent.click(await screen.findByRole("menuitem", { name: "Inside loop “retry”" }));
  expect((await node("ping")).getAttribute("data-path")).toBe("steps[0].body[1]");
});

it("undo brings a deleted node back and redo deletes it again", async () => {
  openEditor(withSteps([{ id: "ping", kind: "http", url: "http://nas.lan" }, { id: "tell", kind: "notify", text: "x" }]));
  const menu = within(await node("ping")).getByRole("button", { name: "Actions for ping" });
  menu.focus();
  await userEvent.keyboard("{Enter}");
  await userEvent.click(await screen.findByRole("menuitem", { name: "Delete" }));
  await waitFor(() => expect(screen.queryByRole("group", { name: "ping" })).toBeNull());
  document.body.focus();
  await userEvent.keyboard("{Control>}z{/Control}");
  expect(await node("ping")).toHaveAttribute("data-path", "steps[0]");
  await userEvent.keyboard("{Shift>}{Control>}z{/Control}{/Shift}");
  await waitFor(() => expect(screen.queryByRole("group", { name: "ping" })).toBeNull());
  await userEvent.click(screen.getByRole("button", { name: "Undo" }));
  expect(await node("ping")).toBeInTheDocument();
});

it("a service field completes an input after {{", async () => {
  openEditor(withSteps([{ id: "check", kind: "probe", service: "" }], { inputs: [plainInput("service")] }));
  pressOnCanvas(await node("check"));
  const service = within(inspector()).getByRole("combobox", { name: "Service" });
  await userEvent.click(service);
  await userEvent.keyboard("{{{{inp");
  const options = within(screen.getByRole("listbox")).getAllByRole("option").map((option) => option.textContent ?? "");
  expect(options.some((option) => option.startsWith("inputs.service"))).toBe(true);
  await userEvent.keyboard("{Enter}");
  expect(service).toHaveValue("{{inputs.service}}");
});

it("saving a new workflow leads to its page, and saving it again updates it without a taken id", async () => {
  const created = { ...sampleWorkflows[1], id: "ping-nas", title: "Ping NAS" };
  const fetch = vi.fn(async (_path: string, init?: RequestInit) => jsonResponse(created, { status: init?.method === "POST" ? 201 : 200, headers: { ETag: '"r2"' } }));
  vi.stubGlobal("fetch", fetch);
  const { client, onSaved } = openEditor(null, {
    initial: { id: "ping-nas", title: "Ping NAS", enabled: true, description: null, tags: [], timeout_seconds: 300, inputs: [], steps: [{ id: "pause", kind: "wait", seconds: 1 }] },
  });
  await userEvent.click(screen.getByRole("button", { name: /^Save$/ }));
  await waitFor(() => expect(onSaved).toHaveBeenCalledWith("ping-nas"));
  expect(fetch).toHaveBeenCalledWith("/api/workflows", expect.objectContaining({ method: "POST" }));
  client.setQueryData(["workflows"], { data: { workflows: [...sampleWorkflows, created] }, revision: '"r2"' });
  pressOnCanvas(await node("pause"));
  const seconds = within(inspector()).getByLabelText(/Seconds/);
  await userEvent.clear(seconds);
  await userEvent.type(seconds, "5");
  await userEvent.click(screen.getByRole("button", { name: /^Save$/ }));
  await waitFor(() => expect(fetch).toHaveBeenCalledWith("/api/workflows/ping-nas", expect.objectContaining({ method: "PUT" })));
  expect(screen.queryByText("This id is taken")).toBeNull();
});

it("the editor offers no run, history or mode control, and Cancel is left to the page", async () => {
  const { onCancel } = openEditor(withSteps([{ id: "ping", kind: "http", url: "http://nas.lan" }]));
  await node("ping");
  const toolbar = screen.getByRole("toolbar", { name: "Editor actions" });
  expect(within(toolbar).queryByRole("button", { name: "Run" })).toBeNull();
  expect(within(toolbar).queryByRole("link", { name: /History/ })).toBeNull();
  expect(screen.queryByRole("radiogroup")).toBeNull();
  await userEvent.click(within(toolbar).getByRole("button", { name: "Cancel" }));
  expect(onCancel).toHaveBeenCalled();
});

it("problems open in the column beside the canvas, and choosing one shows the inspector with the field focused", async () => {
  openEditor(withSteps([{ id: "call", kind: "http", url: "" }]));
  await node("call");
  await userEvent.click(within(screen.getByRole("toolbar", { name: "Editor actions" })).getByRole("button", { name: /^\d+ errors?, / }));
  const column = screen.getByRole("complementary", { name: "Problems" });
  const list = within(column).getByRole("list", { name: "Problems" });
  expect(column.className).not.toContain("absolute");
  await userEvent.click(within(list).getByRole("button", { name: /^call/ }));
  expect(screen.queryByRole("complementary", { name: "Problems" })).toBeNull();
  await waitFor(() => expect(within(inspector()).getByRole("combobox", { name: "URL" })).toHaveFocus());
});

it("the start node keeps its icon's size whatever the length of the title", async () => {
  openEditor(withSteps([{ id: "ping", kind: "http", url: "http://nas.lan" }], { title: "A very long workflow title that fills the whole start node and more" }));
  const start = await screen.findByRole("group", { name: "Start" });
  expect(start.querySelector("[data-start-icon]")?.className).toContain("shrink-0");
});

it("an input gets a type, a default of that type and a description, and saving writes them", async () => {
  const fetch = vi.fn(async () => jsonResponse(sampleWorkflows[1]));
  vi.stubGlobal("fetch", fetch);
  const { onSaved } = openEditor(withSteps([{ id: "ping", kind: "http", url: "http://nas.lan" }], { inputs: [plainInput("hosts")] }));
  pressOnCanvas(await node("Start"));
  await userEvent.selectOptions(within(inspector()).getByRole("combobox", { name: "Type of input 1" }), "list");
  await userEvent.click(within(inspector()).getByRole("button", { name: "Add a value" }));
  await userEvent.type(within(inspector()).getByLabelText("Value 1"), "nas");
  await userEvent.type(within(inspector()).getByLabelText("Description of input 1"), "Hosts to check");
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() => expect(onSaved).toHaveBeenCalled());
  expect(sentBody(fetch).inputs).toEqual([{ name: "hosts", type: "list", default: ["nas"], description: "Hosts to check" }]);
});

it("a loop offers one mode at a time and saves only the chosen one", async () => {
  const fetch = vi.fn(async () => jsonResponse(sampleWorkflows[1]));
  vi.stubGlobal("fetch", fetch);
  const { onSaved } = openEditor(withSteps([{ id: "list", kind: "http", url: "http://nas.lan/items" }]));
  await addFromSlot(await slot("|steps|1"), "Loop");
  const modes = within(inspector()).getByRole("radiogroup", { name: "How the loop repeats" });
  expect(within(modes).getByRole("radio", { name: "Times" })).toHaveAttribute("aria-checked", "true");
  expect(within(inspector()).queryByRole("combobox", { name: /For each/ })).toBeNull();
  await userEvent.click(within(modes).getByRole("radio", { name: "For each" }));
  expect(within(inspector()).queryByLabelText(/^Times/)).toBeNull();
  fireEvent.change(within(inspector()).getAllByRole("combobox").find((box) => box.getAttribute("aria-label")?.startsWith("For each"))!, { target: { value: "{{steps.list.json}}" } });
  await addFromSlot(await slot("steps[1]|body|0"), "Do nothing");
  expect(screen.getByRole("button", { name: "Save" })).not.toHaveTextContent(/\d/);
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() => expect(onSaved).toHaveBeenCalled());
  const loop = sentBody(fetch).steps[1];
  expect(loop.for_each).toBe("{{steps.list.json}}");
  expect(loop).not.toHaveProperty("repeat");
  expect(loop).not.toHaveProperty("while");
});

it("a set node without a stray sign reads as a list of two", async () => {
  openEditor(withSteps([{ id: "services", kind: "set", variable: "svc", list: ["{{portal.services.media.name}}", "{{portal.services.nas.name}}"] }]));
  const set = await node("services");
  expect(set).toHaveTextContent("svc: list of 2");
  expect(set.textContent).not.toMatch(/svc =/);
});

it("a background refresh does not change the revision it sends, and a conflict can be overwritten", async () => {
  const puts = () => fetch.mock.calls.filter(([, sent]) => sent?.method === "PUT");
  const fetch = vi.fn(async (_path: string, init?: RequestInit) => (init?.method === "PUT" && puts().length === 1 ? new Response("stale", { status: 409 }) : jsonResponse(sampleWorkflows[0])));
  vi.stubGlobal("fetch", fetch);
  const { refreshed, onSaved } = openEditor(sampleWorkflows[0]);
  refreshed('"r2"');
  await userEvent.click(screen.getByRole("button", { name: /^Save$/ }));
  expect(await screen.findByRole("button", { name: "Reload" })).toBeInTheDocument();
  expect(onSaved).not.toHaveBeenCalled();
  await userEvent.click(screen.getByRole("button", { name: "Overwrite" }));
  await waitFor(() => expect(onSaved).toHaveBeenCalled());
  expect(puts().map(([, sent]) => (sent?.headers as Record<string, string>)["If-Match"])).toEqual(['"r1"', '"r2"']);
});

it("saving fills an empty branch with a step that does nothing, shows it and leaves nothing unsaved", async () => {
  const fetch = vi.fn(async () => jsonResponse(sampleWorkflows[1]));
  vi.stubGlobal("fetch", fetch);
  const { onSaved } = openEditor(withSteps([{ id: "check", kind: "if", condition: { left: "{{inputs.target}}", op: "==", right: "nas" }, then: [{ id: "done", kind: "stop", outcome: "succeeded" }] }], { inputs: [plainInput("target")] }));
  await userEvent.click(screen.getByRole("button", { name: /^Save$/ }));
  await waitFor(() => expect(onSaved).toHaveBeenCalled());
  expect(sentBody(fetch).steps[0].else).toEqual([{ id: "nothing", kind: "nothing" }]);
  expect(await node("nothing")).toHaveTextContent("Do nothing");
  const leaving = new Event("beforeunload", { cancelable: true });
  window.dispatchEvent(leaving);
  expect(leaving.defaultPrevented).toBe(false);
});

it("an empty workflow is still refused and gets no step that does nothing", async () => {
  const fetch = vi.fn();
  vi.stubGlobal("fetch", fetch);
  openEditor(null, { initial: { id: "empty", title: "Empty", enabled: true, description: null, tags: [], timeout_seconds: 300, inputs: [], steps: [] } });
  await userEvent.click(screen.getByRole("button", { name: /^Save/ }));
  expect(await screen.findByText("Add at least one step")).toBeInTheDocument();
  expect(fetch).not.toHaveBeenCalled();
});
