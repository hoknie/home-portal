import { fireEvent, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, expect, it, vi } from "vitest";

import { hiddenFields, idsOf, newStep, plainInput } from "@/entities/workflow";
import { dictionaries } from "@/shared/i18n";
import { jsonResponse } from "@/shared/lib/testing";

import { NESTED_TYPES } from "./step-form";
import { catalogue, inspector, node, openEditor, pressOnCanvas, sampleWorkflows, sentBody, stubCanvasDom, withSteps } from "../testing-support";

vi.mock("next/navigation", () => ({ useRouter: () => ({ push: vi.fn(), replace: vi.fn() }) }));

const labels = dictionaries.en.workflowHelp.fields as Record<string, Record<string, { label: string }>>;

beforeEach(() => {
  stubCanvasDom();
});

afterEach(() => {
  vi.unstubAllGlobals();
});

it("every catalogue kind opens a form with a control for each of its fields", async () => {
  const taken = new Set<string>();
  const steps = catalogue.kinds.map((kind) => {
    const step = newStep(kind, taken);
    idsOf([step]).forEach((id) => taken.add(id));
    return step;
  });
  openEditor(withSteps(steps));
  for (const kind of catalogue.kinds) {
    const step = steps.find((entry) => entry.kind === kind.name)!;
    pressOnCanvas(await node(step.id));
    const panel = await waitFor(() => inspector());
    const hidden = hiddenFields(step, kind);
    for (const field of kind.fields.filter((entry) => !NESTED_TYPES.includes(entry.type) && !hidden.has(entry.name))) {
      const label = labels[kind.name][field.name].label;
      expect(within(panel).getAllByText(label, { exact: false }).length, `${kind.name}.${field.name}`).toBeGreaterThan(0);
    }
  }
});

it("typing {{ inside a for_each loop offers loop values and earlier results, not later steps", async () => {
  openEditor(
    withSteps([
      { id: "list", kind: "http", url: "http://nas.lan/disks" },
      { id: "each", kind: "loop", for_each: "{{steps.list.json}}", body: [{ id: "tell", kind: "notify", text: "" }] },
      { id: "after", kind: "notify", text: "" },
    ]),
  );
  pressOnCanvas(await node("tell"));
  const text = within(inspector()).getByRole("combobox", { name: "Text" });
  await userEvent.click(text);
  await userEvent.keyboard("{{{{");
  const options = within(screen.getByRole("listbox")).getAllByRole("option").map((option) => option.textContent ?? "");
  expect(options.some((option) => option.startsWith("loop.item"))).toBe(true);
  expect(options.some((option) => option.startsWith("loop.index"))).toBe(true);
  expect(options.some((option) => option.startsWith("steps.list.json"))).toBe(true);
  expect(options.some((option) => option.startsWith("steps.after"))).toBe(false);
  expect(within(screen.getByRole("listbox")).getByRole("group", { name: "Loop" })).toBeInTheDocument();
});

it("a condition built from rows and a group round-trips through saving, with known values on the right", async () => {
  const fetch = vi.fn(async () => jsonResponse(sampleWorkflows[1]));
  vi.stubGlobal("fetch", fetch);
  const { onSaved } = openEditor(
    withSteps([
      { id: "probe", kind: "probe", service: "nas" },
      { id: "check", kind: "if", condition: { left: "{{steps.probe.state}}", op: "==", right: "" }, then: [{ id: "done", kind: "stop", outcome: "succeeded" }] },
    ]),
  );
  pressOnCanvas(await node("check"));
  const panel = inspector();
  await userEvent.click(within(panel).getByRole("button", { name: "down" }));
  await userEvent.click(within(panel).getByRole("button", { name: "Add group" }));
  const lefts = within(panel).getAllByRole("combobox", { name: "Value" });
  await userEvent.type(lefts[1], "b");
  await userEvent.selectOptions(within(panel).getAllByRole("combobox", { name: "Operator" })[1], "is-empty");
  await userEvent.selectOptions(within(panel).getAllByRole("combobox", { name: "How rows combine" })[0], "any");
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() => expect(onSaved).toHaveBeenCalled());
  expect(sentBody(fetch).steps[1].condition).toEqual({ any: [{ left: "{{steps.probe.state}}", op: "==", right: "down" }, { any: [{ left: "b", op: "is-empty" }] }] });
});

it("what a step gives is listed with its meaning and copies its reference", async () => {
  const writeText = vi.fn(async () => undefined);
  Object.defineProperty(navigator, "clipboard", { configurable: true, value: { writeText } });
  openEditor(withSteps([{ id: "ping", kind: "http", url: "http://nas.lan" }]));
  pressOnCanvas(await node("ping"));
  const produces = within(inspector()).getByRole("region", { name: "What this step gives" });
  expect(within(produces).getByText("The HTTP status code, such as 200 or 404")).toBeInTheDocument();
  await userEvent.click(within(produces).getByRole("button", { name: "Copy {{steps.ping.status}}" }));
  expect(writeText).toHaveBeenCalledWith("{{steps.ping.status}}");
});

it("an answer structure typed on an http step feeds the key suggestions of later steps", async () => {
  openEditor(
    withSteps([
      { id: "ping", kind: "http", url: "http://nas.lan" },
      { id: "tell", kind: "notify", text: "" },
    ]),
  );
  pressOnCanvas(await node("ping"));
  const structure = within(inspector()).getByRole("region", { name: "Answer structure" });
  fireEvent.change(within(structure).getByRole("textbox"), { target: { value: '{"state":"up","disks":[{"name":"sda"}]}' } });
  expect(within(structure).getByText("state")).toBeInTheDocument();
  const long = "living_room_ceiling_light_second_floor_display_name";
  fireEvent.change(within(structure).getByRole("textbox"), { target: { value: JSON.stringify({ [long]: "Ceiling light above the sofa by the window, second floor" }) } });
  const chip = structure.querySelector(`[data-key="${long}"]`) as HTMLElement;
  expect(within(chip).getByText(long)).toHaveClass("break-all");
  expect(within(chip).getByTitle(/^Ceiling light above the sofa/)).toHaveClass("truncate");
  fireEvent.change(within(structure).getByRole("textbox"), { target: { value: "<html>" } });
  expect(within(structure).getByRole("alert")).toHaveTextContent("This is not JSON");
  fireEvent.change(within(structure).getByRole("textbox"), { target: { value: '{"state":"up"}' } });
  pressOnCanvas(await node("tell"));
  await userEvent.click(within(inspector()).getByRole("combobox", { name: "Text" }));
  await userEvent.keyboard("{{{{steps.ping.json.");
  const options = within(screen.getByRole("listbox")).getAllByRole("option").map((option) => option.textContent ?? "");
  expect(options.some((option) => option.startsWith("steps.ping.json.state") && option.includes("up"))).toBe(true);
});

it("a branch can do nothing on purpose", async () => {
  openEditor(withSteps([{ id: "check", kind: "if", condition: { left: "a", op: "==", right: "a" }, then: [], else: [{ id: "tell", kind: "notify", text: "x" }] }]));
  const slot = await waitFor(() => {
    const found = document.querySelector<HTMLElement>('[data-slot="steps[0]|then|0"]');
    expect(found).not.toBeNull();
    return found as HTMLElement;
  });
  pressOnCanvas(slot);
  const dialog = await screen.findByRole("dialog", { name: "Add a step" });
  await userEvent.click(within(dialog).getByRole("button", { name: /^Do nothing/ }));
  expect(await node("nothing")).toHaveTextContent("goes on");
});

it("an old telegram step opens as a notification to telegram, and the channel picker marks channels not set up", async () => {
  openEditor(withSteps([{ id: "tell", kind: "telegram", text: "x" }]));
  pressOnCanvas(await node("tell"));
  const channel = within(inspector()).getByRole("combobox", { name: "Channel" });
  expect(channel).toHaveValue("telegram");
  await userEvent.clear(channel);
  await userEvent.click(channel);
  const options = within(screen.getByRole("listbox")).getAllByRole("option");
  expect(options.map((option) => option.textContent ?? "")).toEqual([expect.stringContaining("telegram"), expect.stringContaining("Not set up")]);
  expect(options[1]).toHaveAttribute("aria-disabled", "true");
});

it("typing a bar after a list offers list filters first and not text filters", async () => {
  openEditor(
    withSteps([
      { id: "list", kind: "http", url: "http://nas.lan", response_sample: '{"disks": [{"name": "sda"}, {"name": "sdb"}]}' },
      { id: "tell", kind: "notify", text: "" },
    ]),
  );
  pressOnCanvas(await node("tell"));
  const text = within(inspector()).getByRole("combobox", { name: "Text" });
  await userEvent.click(text);
  await userEvent.keyboard("{{{{steps.list.json.disks |");
  const offered = within(screen.getByRole("listbox")).getAllByRole("option").map((option) => option.textContent ?? "");
  expect(offered.slice(0, 3).map((label) => label.split(/[(\s]/)[0])).toEqual(expect.arrayContaining(["slice"]));
  expect(offered.some((label) => label.startsWith("length"))).toBe(true);
  expect(offered.some((label) => label.startsWith("join(separator)"))).toBe(true);
  expect(offered.some((label) => label.startsWith("pluck(key)"))).toBe(true);
  expect(offered.some((label) => label.startsWith("upper"))).toBe(false);
  expect(offered.find((label) => label.startsWith("length"))).toContain("2");
  await userEvent.keyboard("pl{Enter}");
  expect(text).toHaveValue('{{steps.list.json.disks | pluck("key")');
});

it("a dictionary is built from rows without JSON and its keys are offered to later steps", async () => {
  const fetch = vi.fn(async () => jsonResponse(sampleWorkflows[1]));
  vi.stubGlobal("fetch", fetch);
  const { onSaved } = openEditor(withSteps([{ id: "report", kind: "set", variable: "report", value: "" }, { id: "tell", kind: "notify", text: "" }], { inputs: [plainInput("service")] }));
  pressOnCanvas(await node("report"));
  await userEvent.click(within(inspector()).getByRole("radio", { name: "Dictionary" }));
  const key = within(inspector()).getByRole("combobox", { name: "Dictionary: name 1" });
  fireEvent.change(key, { target: { value: "host" } });
  fireEvent.change(within(inspector()).getByRole("combobox", { name: "Value of host" }), { target: { value: "{{inputs.service}}" } });
  pressOnCanvas(await node("tell"));
  const text = within(inspector()).getByRole("combobox", { name: "Text" });
  await userEvent.click(text);
  await userEvent.keyboard("{{{{vars.report.");
  const offered = within(screen.getByRole("listbox")).getAllByRole("option").map((option) => option.textContent ?? "");
  expect(offered.some((option) => option.startsWith("vars.report.host"))).toBe(true);
  await userEvent.keyboard("{Escape}");
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() => expect(onSaved).toHaveBeenCalled());
  expect(sentBody(fetch).steps[0]).toEqual({ id: "report", kind: "set", variable: "report", object: { host: "{{inputs.service}}" } });
});

it("a script step takes named variables, marks a bad name before saving, and writes them", async () => {
  const fetch = vi.fn(async () => jsonResponse(sampleWorkflows[1]));
  vi.stubGlobal("fetch", fetch);
  const { onSaved } = openEditor(withSteps([{ id: "run", kind: "script", script: "restart.sh" }], { inputs: [plainInput("service")] }));
  pressOnCanvas(await node("run"));
  const add = within(inspector()).getAllByRole("button", { name: "Add" }).find((button) => button.closest("div")?.textContent?.includes("Environment")) ?? within(inspector()).getAllByRole("button", { name: /Add/ })[1];
  await userEvent.click(add);
  fireEvent.change(within(inspector()).getByRole("combobox", { name: "Environment: name 1" }), { target: { value: "STEP_TARGET" } });
  fireEvent.change(within(inspector()).getByRole("combobox", { name: "Value of STEP_TARGET" }), { target: { value: "{{inputs.service}}" } });
  await userEvent.click(add);
  fireEvent.change(within(inspector()).getByRole("combobox", { name: "Environment: name 2" }), { target: { value: "path" } });
  expect(within(inspector()).getByText("STEP_ followed by capital letters, digits and _")).toBeInTheDocument();
  fireEvent.change(within(inspector()).getByRole("combobox", { name: "Environment: name 2" }), { target: { value: "STEP_MODE" } });
  fireEvent.change(within(inspector()).getByRole("combobox", { name: "Standard input" }), { target: { value: "{{inputs.service}}" } });
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() => expect(onSaved).toHaveBeenCalled());
  expect(sentBody(fetch).steps[0]).toMatchObject({ env: { STEP_TARGET: "{{inputs.service}}" }, stdin: "{{inputs.service}}" });
});

it("an automation step picks an automation, suggests its event fields and shows its title on the node", async () => {
  const fetch = vi.fn(async () => jsonResponse(sampleWorkflows[1]));
  vi.stubGlobal("fetch", fetch);
  const { onSaved } = openEditor(withSteps([{ id: "go", kind: "automation", automation: "" }], { inputs: [plainInput("service")] }));
  pressOnCanvas(await node("go"));
  const picker = within(inspector()).getByRole("combobox", { name: "Automation" });
  await userEvent.click(picker);
  const options = within(screen.getByRole("listbox")).getAllByRole("option");
  expect(options.map((option) => option.textContent ?? "")).toEqual([expect.stringContaining("Restart media"), expect.stringContaining("Sleeping")]);
  expect(options[1]).toHaveAttribute("aria-disabled", "true");
  await userEvent.click(options[0]);
  await userEvent.click(within(inspector()).getAllByRole("button", { name: "Add" }).at(-1)!);
  const key = within(inspector()).getByRole("combobox", { name: "Event fields: name 1" });
  await userEvent.click(key);
  expect(within(screen.getByRole("listbox")).getAllByRole("option").some((option) => option.textContent?.startsWith("service.id"))).toBe(true);
  fireEvent.change(key, { target: { value: "service.id" } });
  fireEvent.change(within(inspector()).getByRole("combobox", { name: "Value of service.id" }), { target: { value: "{{inputs.service}}" } });
  await userEvent.click(within(inspector()).getByRole("switch", { name: /Wait for it/ }));
  expect(await node("go")).toHaveTextContent("Restart media, waiting");
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() => expect(onSaved).toHaveBeenCalled());
  expect(sentBody(fetch).steps[0]).toEqual({ id: "go", kind: "automation", automation: "restart-media", fields: { "service.id": "{{inputs.service}}" }, wait: true });
});

it("after split an element filter such as upper is offered, marked as applying to each element", async () => {
  openEditor(withSteps([{ id: "tell", kind: "notify", text: "" }], { inputs: [plainInput("names")] }));
  pressOnCanvas(await node("tell"));
  const text = within(inspector()).getByRole("combobox", { name: "Text" });
  await userEvent.click(text);
  await userEvent.keyboard('{{{{inputs.names | split(",") |');
  const upper = within(screen.getByRole("listbox"))
    .getAllByRole("option")
    .find((option) => option.textContent?.startsWith("upper"));
  expect(upper?.textContent).toContain("for each element");
});

it("condition operators are named in words, on the form and on the node", async () => {
  openEditor(withSteps([{ id: "check", kind: "if", condition: { left: "{{steps.x}}", op: "!=", right: "ok" }, then: [] }]));
  expect(await node("check")).toHaveTextContent("does not equal ok");
  pressOnCanvas(await node("check"));
  const operator = within(inspector()).getByRole("combobox", { name: "Operator" });
  expect(within(operator).getAllByRole("option").map((option) => option.textContent)).toEqual([
    "equals",
    "does not equal",
    "is less than",
    "is at most",
    "is greater than",
    "is at least",
    "contains",
    "is empty",
    "is not empty",
  ]);
  expect(operator).toHaveValue("!=");
});

it("no kind's inspector shows two fields of one exclusive group", async () => {
  const taken = new Set<string>();
  const kinds = catalogue.kinds.filter((kind) => kind.exclusive.length > 0);
  const steps = kinds.map((kind) => {
    const step = newStep(kind, taken);
    taken.add(step.id);
    return step;
  });
  openEditor(withSteps(steps));
  for (const [index, kind] of kinds.entries()) {
    pressOnCanvas(await node(steps[index].id));
    const panel = await waitFor(() => inspector());
    for (const group of kind.exclusive) {
      const shown = group.filter((field) =>
        within(panel)
          .queryAllByText(labels[kind.name][field].label)
          .some((element) => element.closest("[role=radio]") === null),
      );
      expect(shown, kind.name).toHaveLength(1);
    }
  }
});

it("typing {{portal.services.me offers the service's id, name and state with their current values", async () => {
  openEditor(withSteps([{ id: "tell", kind: "notify", text: "" }]));
  pressOnCanvas(await node("tell"));
  await userEvent.click(within(inspector()).getByRole("combobox", { name: "Text" }));
  await userEvent.keyboard("{{{{portal.services.me");
  const options = within(screen.getByRole("listbox")).getAllByRole("option").map((option) => option.textContent ?? "");
  expect(options.find((option) => option.includes("portal.services.media.id"))).toContain("media");
  expect(options.find((option) => option.includes("portal.services.media.name"))).toContain("Media");
  expect(options.some((option) => option.includes("portal.services.media.state"))).toBe(true);
  expect(options.some((option) => option.includes("portal.network."))).toBe(false);
});

it("a dictionary row keyed by a variable offers the loop's names in its key and saves the template as the key", async () => {
  const fetch = vi.fn(async () => jsonResponse(sampleWorkflows[1]));
  vi.stubGlobal("fetch", fetch);
  openEditor(withSteps([{ id: "each", kind: "loop", for_each: "{{portal.services}}", body: [{ id: "states", kind: "set", variable: "states", object: { name: "x" } }] }]));
  pressOnCanvas(await node("states"));
  const key = within(inspector()).getByRole("combobox", { name: "Dictionary: name 1" });
  fireEvent.change(key, { target: { value: "" } });
  await userEvent.click(key);
  await userEvent.keyboard("{{{{loop.it");
  const options = within(screen.getByRole("listbox")).getAllByRole("option").map((option) => option.textContent ?? "");
  expect(options.some((option) => option.startsWith("loop.item"))).toBe(true);
  fireEvent.change(key, { target: { value: "{{loop.item.id}}" } });
  await userEvent.click(screen.getByRole("button", { name: /^Save( \(|$)/ }));
  await waitFor(() => expect(fetch).toHaveBeenCalled());
  expect(sentBody(fetch).steps[0].body[0].object).toEqual({ "{{loop.item.id}}": "x" });
});

it("a filter argument completed from the scope inserts the name without quotes", async () => {
  openEditor(withSteps([{ id: "states", kind: "http", url: "http://nas.lan" }, { id: "each", kind: "loop", for_each: "{{steps.states.json}}", body: [{ id: "tell", kind: "notify", text: "" }] }]));
  pressOnCanvas(await node("tell"));
  const text = within(inspector()).getByRole("combobox", { name: "Text" });
  await userEvent.click(text);
  await userEvent.keyboard("{{{{steps.states.json | get(lo");
  const options = within(screen.getByRole("listbox")).getAllByRole("option").map((option) => option.textContent ?? "");
  expect(options.some((option) => option.startsWith("loop.item"))).toBe(true);
  expect(options.some((option) => option.startsWith("loop.index"))).toBe(true);
  await userEvent.keyboard("{Enter}");
  expect(text).toHaveValue("{{steps.states.json | get(loop.item");
});

it("a number from an input is a template checked when the step runs, and a plain number is still bounded", async () => {
  const fetch = vi.fn(async () => jsonResponse(sampleWorkflows[1]));
  vi.stubGlobal("fetch", fetch);
  openEditor(withSteps([{ id: "nap", kind: "wait", seconds: 5 }], { inputs: [plainInput("pause")] }));
  pressOnCanvas(await node("nap"));
  const seconds = within(inspector()).getByRole("combobox", { name: "Seconds" });
  fireEvent.change(seconds, { target: { value: "5000" } });
  expect(await within(inspector()).findByText("Outside the allowed range")).toBeInTheDocument();
  expect(seconds).toHaveAttribute("aria-invalid", "true");
  fireEvent.change(seconds, { target: { value: "{{inputs.pause}}" } });
  expect(within(inspector()).getByText("A template: checked when the step runs")).toBeInTheDocument();
  await userEvent.click(screen.getByRole("button", { name: /^Save( \(|$)/ }));
  await waitFor(() => expect(fetch).toHaveBeenCalled());
  expect(sentBody(fetch).steps[0].seconds).toBe("{{inputs.pause}}");
});

it("a script that may fail: Continue is off by default, on writes fail_on_error false, off again drops the key, as for http", async () => {
  const fetch = vi.fn(async () => jsonResponse(sampleWorkflows[1]));
  vi.stubGlobal("fetch", fetch);
  const { onSaved } = openEditor(
    withSteps([
      { id: "run", kind: "script", script: "restart.sh" },
      { id: "ping", kind: "http", url: "http://nas.lan", fail_on_error: false },
    ]),
  );
  pressOnCanvas(await node("run"));
  const going = within(inspector()).getByRole("switch", { name: "Continue the run when the script fails" });
  expect(going).not.toBeChecked();
  await userEvent.click(going);
  pressOnCanvas(await node("ping"));
  await userEvent.click(within(inspector()).getByRole("switch", { name: "Fail on status 400 and above" }));
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() => expect(onSaved).toHaveBeenCalledTimes(1));
  let steps = sentBody(fetch).steps;
  expect(steps[0].fail_on_error).toBe(false);
  expect("fail_on_error" in steps[1]).toBe(false);
  pressOnCanvas(await node("run"));
  const again = within(inspector()).getByRole("switch", { name: "Continue the run when the script fails" });
  expect(again).toBeChecked();
  await userEvent.click(again);
  fetch.mockClear();
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() => expect(fetch).toHaveBeenCalled());
  steps = sentBody(fetch).steps;
  expect("fail_on_error" in steps[0]).toBe(false);
});
