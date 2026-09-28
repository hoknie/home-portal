import { fireEvent, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, expect, it, vi } from "vitest";

import { jsonResponse } from "@/shared/lib/testing";

import { addFromSlot, inspector, openEditor, sampleWorkflows, sentBody, stubCanvasDom, withSteps } from "../../testing-support";

vi.mock("next/navigation", () => ({ useRouter: () => ({ push: vi.fn(), replace: vi.fn() }) }));

beforeEach(() => {
  stubCanvasDom();
});

afterEach(() => {
  vi.unstubAllGlobals();
});

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

it("building the unhealthy disks list previews each operation and saves them in order", async () => {
  const fetch = vi.fn(async () => jsonResponse(sampleWorkflows[1]));
  vi.stubGlobal("fetch", fetch);
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
  expect(within(card(1)).getByRole("figure", { name: "After operation 1" })).toHaveTextContent('"name":"sdb"');
  expect(within(card(1)).getByRole("figure", { name: "After operation 1" })).not.toHaveTextContent('"name":"sda"');

  await addOperation("pluck");
  fireEvent.change(within(card(2)).getByRole("combobox", { name: "key" }), { target: { value: "name" } });
  expect(within(card(2)).getByRole("figure", { name: "After operation 2" })).toHaveTextContent('["sdb","sdc"]');

  await addOperation("join");
  await userEvent.type(within(card(3)).getByLabelText("separator"), ", ");
  const last = within(card(3)).getByRole("figure", { name: "After operation 3" });
  expect(last).toHaveTextContent('"sdb, sdc"');
  expect(last).toHaveTextContent("text");
  expect(within(inspector()).getByRole("region", { name: "What this step gives" })).toHaveTextContent("text");

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
  openEditor(
    withSteps([
      { id: "list", kind: "http", url: "http://nas.lan/disks", response_sample: DISKS },
      { id: "bad", kind: "transform", input: "{{steps.list.json.disks}}", operations: [{ op: "upper" }, { op: "length" }] },
    ]),
  );
  fireEvent.click(await screen.findByRole("group", { name: "bad" }));
  expect(card(1)).toHaveAttribute("data-failed", "true");
  expect(within(card(1)).getByRole("alert")).toHaveTextContent("upper takes text and got an object at position 0");
  await userEvent.click(within(card(2)).getByRole("button", { name: "Move up" }));
  expect(within(card(1)).getByRole("heading")).toHaveTextContent("length");
  expect(within(card(1)).getByRole("figure", { name: "After operation 1" })).toHaveTextContent("3");
});

it("each runs a nested chain whose cards preview the first element, and saves the nested operations", async () => {
  const fetch = vi.fn(async () => jsonResponse(sampleWorkflows[1]));
  vi.stubGlobal("fetch", fetch);
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
  const inner = within(nested).getAllByRole("listitem");
  expect(within(inner[1]).getByRole("figure", { name: "After operation 2" })).toHaveTextContent('"NAS"');
  expect(within(card(1)).getAllByRole("figure", { name: "After operation 1" }).at(-1)).toHaveTextContent('["NAS","ROUTER"]');
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() => expect(onSaved).toHaveBeenCalled());
  expect(sentBody(fetch).steps[1].operations).toEqual([{ op: "each", operations: [{ op: "trim" }, { op: "upper" }] }]);
});

it("after split the picker offers what fits a list first, trim for each element, and the preview trims every item", async () => {
  openEditor(
    withSteps([
      { id: "list", kind: "http", url: "http://nas.lan/names", response_sample: '{"names": "nas , router"}' },
      { id: "clean", kind: "transform", input: "{{steps.list.json.names}}", operations: [{ op: "split", args: [","] }] },
    ]),
  );
  fireEvent.click(await screen.findByRole("group", { name: "clean" }));
  expect(within(inspector()).getByText("The value now: list")).toBeInTheDocument();
  const picker = within(inspector()).getAllByRole("combobox", { name: "Operation to add" }).at(-1)!;
  const fitting = within(picker).getByRole("group", { name: "Fits the value now (list)" });
  const labels = within(fitting).getAllByRole("option").map((option) => option.textContent);
  expect(labels).toContain("trim — for each element");
  expect(labels).toContain("join");
  await userEvent.selectOptions(picker, within(fitting).getByRole("option", { name: "trim — for each element" }));
  await userEvent.click(within(inspector()).getAllByRole("button", { name: "Add operation" }).at(-1)!);
  expect(within(card(2)).getByRole("figure", { name: "After operation 2" })).toHaveTextContent('["nas","router"]');
});
