import { fireEvent, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, expect, it, vi } from "vitest";

import { jsonResponse } from "@/shared/lib/testing";

import { Failing, addFromSlot, inspector, node, openEditor, pressOnCanvas, previewRequests, sampleWorkflows, sentBody, stubCanvasDom, stubPreviews, withSteps } from "../../testing-support";

vi.mock("next/navigation", () => ({ useRouter: () => ({ push: vi.fn(), replace: vi.fn() }) }));

beforeEach(() => {
  stubCanvasDom();
});

afterEach(() => {
  vi.unstubAllGlobals();
});

const FAILING = [
  { name: "sdb", health: "failing" },
  { name: "sdc", health: "failing" },
];

async function shown(item: () => HTMLElement, figure: string, text: string) {
  await waitFor(() => expect(within(item()).getAllByRole("figure", { name: figure }).at(-1)).toHaveTextContent(text));
}

const DISKS = '{"disks": [{"name": "sda", "health": "ok"}, {"name": "sdb", "health": "failing"}, {"name": "sdc", "health": "failing"}]}';

async function slot(key: string) {
  return waitFor(() => {
    const found = document.querySelector<HTMLElement>(`[data-slot="${key}"]`);
    expect(found, key).not.toBeNull();
    return found as HTMLElement;
  });
}

function card(number: number) {
  return within(inspector()).getAllByRole("listitem", { name: new RegExp(`^${number}\\. `) })[0];
}

async function addOperation(name: string) {
  await userEvent.selectOptions(within(inspector()).getByRole("combobox", { name: "Operation to add" }), name);
  await userEvent.click(within(inspector()).getByRole("button", { name: "Add operation" }));
}

it("building the unhealthy disks list previews each operation from the portal and saves them in order", async () => {
  const fetch = stubPreviews({
    filter: { steps: [FAILING] },
    "filter,pluck": { steps: [FAILING, ["sdb", "sdc"]] },
    "filter,pluck,join": { steps: [FAILING, ["sdb", "sdc"], "sdb, sdc"] },
  });
  const { onSaved } = openEditor(withSteps([{ id: "list", kind: "http", url: "http://nas.lan/disks", response_sample: DISKS }]));
  await addFromSlot(await slot("|steps|1"), "Transform");
  const input = within(inspector()).getByRole("combobox", { name: "Input" });
  await userEvent.click(input);
  await userEvent.keyboard("{{{{steps.list.json.dis");
  await userEvent.keyboard("{Enter}");
  expect(input).toHaveValue("{{steps.list.json.disks}}");

  await addOperation("filter");
  fireEvent.change(within(card(1)).getByRole("combobox", { name: "Value" }), { target: { value: "{{item.health}}" } });
  await userEvent.selectOptions(within(card(1)).getByRole("combobox", { name: "Operator" }), "!=");
  fireEvent.change(within(card(1)).getByRole("combobox", { name: "Compared with" }), { target: { value: "ok" } });
  await shown(() => card(1), "After operation 1", '"name":"sdb"');
  expect(within(card(1)).getByRole("figure", { name: "After operation 1" })).not.toHaveTextContent('"name":"sda"');

  await addOperation("pluck");
  fireEvent.change(within(card(2)).getByRole("combobox", { name: "key" }), { target: { value: "name" } });
  await shown(() => card(2), "After operation 2", '["sdb","sdc"]');

  await addOperation("join");
  await userEvent.type(within(card(3)).getByLabelText("separator"), ", ");
  await shown(() => card(3), "After operation 3", '"sdb, sdc"');
  expect(within(card(3)).getByRole("figure", { name: "After operation 3" })).toHaveTextContent("text");
  expect(within(inspector()).getByRole("region", { name: "What this step gives" })).toHaveTextContent("text");
  const asked = previewRequests(fetch).at(-1)!;
  expect(asked.value).toEqual(JSON.parse(DISKS).disks);
  expect(asked.operations).toEqual([
    { op: "filter", where: { left: "{{item.health}}", op: "!=", right: "ok" } },
    { op: "pluck", args: ["name"] },
    { op: "join", args: [", "] },
  ]);

  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() => expect(onSaved).toHaveBeenCalled());
  expect(sentBody(fetch).steps[1]).toMatchObject({
    kind: "transform",
    input: "{{steps.list.json.disks}}",
    operations: [
      { op: "filter", where: { left: "{{item.health}}", op: "!=", right: "ok" } },
      { op: "pluck", args: ["name"] },
      { op: "join", args: [", "] },
    ],
  });
});

it("an operation that cannot take the value is marked, and cards move up and down", async () => {
  const refusal = new Failing("operation 1 (upper): upper takes text and got an object at position 0");
  stubPreviews({ "upper,length": { steps: [refusal] }, "length,upper": { steps: [3, new Failing("operation 2 (upper): upper takes text and got a number")] } });
  openEditor(
    withSteps([
      { id: "list", kind: "http", url: "http://nas.lan/disks", response_sample: DISKS },
      { id: "bad", kind: "transform", input: "{{steps.list.json.disks}}", operations: [{ op: "upper" }, { op: "length" }] },
    ]),
  );
  fireEvent.click(await screen.findByRole("group", { name: "bad" }));
  await waitFor(() => expect(card(1)).toHaveAttribute("data-failed", "true"));
  expect(within(card(1)).getByRole("alert")).toHaveTextContent("upper takes text and got an object at position 0");
  await userEvent.click(within(card(2)).getByRole("button", { name: "Move up" }));
  expect(within(card(1)).getByRole("heading")).toHaveTextContent("length");
  await shown(() => card(1), "After operation 1", "3");
});

it("each runs a nested chain whose cards preview the first element, and saves the nested operations", async () => {
  const fetch = stubPreviews({
    "trim,upper": { steps: ["nas", "NAS"] },
    "each(trim,upper)": { steps: [["NAS", "ROUTER"]] },
  });
  const { onSaved } = openEditor(
    withSteps([
      { id: "list", kind: "http", url: "http://nas.lan/names", response_sample: '{"names": [" nas ", " router "]}' },
      { id: "clean", kind: "transform", input: "{{steps.list.json.names}}", operations: [] },
    ]),
  );
  fireEvent.click(await screen.findByRole("group", { name: "clean" }));
  await addOperation("each");
  const nested = within(card(1)).getByRole("list", { name: "Operations inside, level 1" });
  const nestedAdd = (name: string) => async () => {
    await userEvent.selectOptions(within(card(1)).getByRole("combobox", { name: "Operation to add" }), name);
    await userEvent.click(within(card(1)).getByRole("button", { name: "Add operation" }));
  };
  await nestedAdd("trim")();
  await nestedAdd("upper")();
  await shown(() => within(nested).getAllByRole("listitem")[1], "After operation 2", '"NAS"');
  await shown(() => card(1), "After operation 1", '["NAS","ROUTER"]');
  expect(previewRequests(fetch).find((asked) => asked.operations.length === 2)?.value).toBe(" nas ");
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() => expect(onSaved).toHaveBeenCalled());
  expect(sentBody(fetch).steps[1].operations).toEqual([{ op: "each", operations: [{ op: "trim" }, { op: "upper" }] }]);
});

it("after split the picker offers what fits a list first, trim for each element, and the preview trims every item", async () => {
  stubPreviews({ split: { steps: [["nas ", " router"]] }, "split,trim": { steps: [["nas ", " router"], ["nas", "router"]] } });
  openEditor(
    withSteps([
      { id: "list", kind: "http", url: "http://nas.lan/names", response_sample: '{"names": "nas , router"}' },
      { id: "clean", kind: "transform", input: "{{steps.list.json.names}}", operations: [{ op: "split", args: [","] }] },
    ]),
  );
  fireEvent.click(await screen.findByRole("group", { name: "clean" }));
  expect(await within(inspector()).findByText("The value now: list")).toBeInTheDocument();
  const picker = within(inspector()).getAllByRole("combobox", { name: "Operation to add" }).at(-1)!;
  const fitting = within(picker).getByRole("group", { name: "Fits the value now (list)" });
  const labels = within(fitting).getAllByRole("option").map((option) => option.textContent);
  expect(labels).toContain("trim — for each element");
  expect(labels).toContain("join");
  await userEvent.selectOptions(picker, within(fitting).getByRole("option", { name: "trim — for each element" }));
  await userEvent.click(within(inspector()).getAllByRole("button", { name: "Add operation" }).at(-1)!);
  await shown(() => card(2), "After operation 2", '["nas","router"]');
});

it("an object offers get by key among the filters for objects, with its keys as suggestions and the value in the preview", async () => {
  stubPreviews({ get: { steps: ["up"] } });
  openEditor(withSteps([{ id: "list", kind: "http", url: "http://nas.lan/state", response_sample: '{"state": "up", "uptime": 42}' }]));
  await addFromSlot(await slot("|steps|1"), "Transform");
  fireEvent.change(within(inspector()).getByRole("combobox", { name: "Input" }), { target: { value: "{{steps.list.json}}" } });
  const picker = within(inspector()).getByRole("combobox", { name: "Operation to add" });
  const objects = within(picker).getByRole("group", { name: "Filters for objects" });
  expect(within(objects).getByRole("option", { name: "get by key" })).toBeInTheDocument();
  expect(within(within(picker).getByRole("group", { name: "Filters for lists" })).getByRole("option", { name: "get by key" })).toBeInTheDocument();
  await addOperation("get");
  const key = within(card(1)).getByRole("combobox", { name: "key" });
  await userEvent.click(key);
  expect(within(screen.getByRole("listbox")).getAllByRole("option").map((option) => option.textContent ?? "")).toEqual(expect.arrayContaining([expect.stringContaining("state"), expect.stringContaining("uptime")]));
  fireEvent.change(key, { target: { value: "state" } });
  await shown(() => card(1), "After operation 1", '"up"');
});

it("a later step knows the transformed value's type once the portal answers, and offers the filters that fit it", async () => {
  stubPreviews({ split: { steps: [["nas", "router"]] } });
  openEditor(
    withSteps([
      { id: "list", kind: "http", url: "http://nas.lan/names", response_sample: '{"names": "nas,router"}' },
      { id: "clean", kind: "transform", input: "{{steps.list.json.names}}", operations: [{ op: "split", args: [","] }] },
      { id: "tell", kind: "notify", text: "" },
    ]),
  );
  pressOnCanvas(await node("tell"));
  const text = within(inspector()).getByRole("combobox", { name: "Text" });
  await userEvent.click(text);
  await userEvent.keyboard("{{{{steps.clean.value |");
  await waitFor(() => {
    const offered = within(screen.getByRole("listbox")).getAllByRole("option").map((option) => option.textContent ?? "");
    expect(offered.some((label) => label.startsWith("join(separator)"))).toBe(true);
    expect(offered.some((label) => label.startsWith("upper") && !label.includes("each"))).toBe(false);
  });
});

it("when the portal cannot be reached the cards say so and the step can still be edited and saved", async () => {
  const fetch = vi.fn(async (url: string) => {
    if (url.endsWith("/transform-preview")) {
      throw new TypeError("Failed to fetch");
    }
    return jsonResponse(sampleWorkflows[1]);
  });
  vi.stubGlobal("fetch", fetch);
  const { onSaved } = openEditor(
    withSteps([
      { id: "list", kind: "http", url: "http://nas.lan/disks", response_sample: DISKS },
      { id: "count", kind: "transform", input: "{{steps.list.json.disks}}", operations: [{ op: "length" }] },
    ]),
  );
  fireEvent.click(await screen.findByRole("group", { name: "count" }));
  expect(within(card(1)).getByText("Computing the preview…")).toBeInTheDocument();
  expect(await within(card(1)).findByText("Cannot reach the portal to compute the preview.")).toBeInTheDocument();
  await addOperation("slice");
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() => expect(onSaved).toHaveBeenCalled());
  expect(sentBody(fetch).steps[1].operations.map((operation: { op: string }) => operation.op)).toEqual(["length", "slice"]);
});
