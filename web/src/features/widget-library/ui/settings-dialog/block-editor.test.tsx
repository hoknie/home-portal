import { screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import { afterEach, expect, it, vi } from "vitest";

import { apiSamples } from "@/shared/api";
import { jsonResponse, renderWithProviders } from "@/shared/lib/testing";

import type { RawBlock } from "../../model/blocks";
import { BlockEditor } from "./block-editor";
import { SuggestingContext } from "./template-field";

afterEach(() => vi.unstubAllGlobals());

function edit(start: RawBlock) {
  vi.stubGlobal("fetch", vi.fn(async (input: RequestInfo | URL) => (String(input) === "/api/workflows" ? jsonResponse(apiSamples.workflows, { headers: { ETag: '"w"' } }) : jsonResponse(apiSamples.automations, { headers: { ETag: '"a"' } }))));
  const seen: { block: RawBlock } = { block: start };
  function Harness() {
    const [block, setBlock] = useState(start);
    seen.block = block;
    return (
      <SuggestingContext.Provider value={{ names: (inside) => (inside ? [{ value: "item.name" }] : [{ value: "data.free" }]), filters: () => [] }}>
        <BlockEditor id="b" block={block} errors={{ value: "names {{steps.x}}" }} onChange={setBlock} />
      </SuggestingContext.Provider>
    );
  }
  renderWithProviders(<Harness />);
  return seen;
}

it("a number block edits its texts, shows the error of a field, and switches to thresholds", async () => {
  const seen = edit({ kind: "stat", label: "", value: "{{data}}" });
  await userEvent.type(screen.getByLabelText("Label"), "Free");
  expect(seen.block.label).toBe("Free");
  expect(screen.getByText("names {{steps.x}}")).toBeInTheDocument();
  await userEvent.selectOptions(screen.getByRole("combobox", { name: "Colour" }), "thresholds");
  expect(seen.block.thresholds).toEqual({ warning: 75, danger: 90 });
  expect(seen.block.tone).toBeUndefined();
});

it("a table adds and removes columns, each read for every item", async () => {
  const seen = edit({ kind: "table", items: "{{data.rows}}", columns: [{ header: "Name", value: "{{item.name}}" }] });
  await userEvent.click(screen.getByRole("button", { name: "Add a column" }));
  expect(seen.block.columns).toHaveLength(2);
  await userEvent.click(screen.getAllByRole("button", { name: "Remove" })[0]);
  expect(seen.block.columns).toEqual([{ header: "", value: "{{item}}" }]);
});

it("a button chooses what it does and keeps its id", async () => {
  const seen = edit({ kind: "button", label: "Go", action: { id: "go", refresh: true } });
  await userEvent.selectOptions(screen.getByRole("combobox", { name: "What it does" }), "link");
  expect(seen.block.action).toEqual({ id: "go", link: "https://" });
  await userEvent.selectOptions(screen.getByRole("combobox", { name: "What it does" }), "automation");
  expect((seen.block.action as { automation: string }).automation).toBe("restart-media");
  expect(within(screen.getByRole("group", { name: "What it does" })).getByRole("combobox", { name: "Automation" })).toBeInTheDocument();
});

it("visual choices are icons named by their labels: text size, a fixed colour and the threshold direction", async () => {
  const seen = edit({ kind: "text", text: "hi" });
  await userEvent.click(within(screen.getByRole("radiogroup", { name: "Size" })).getByRole("radio", { name: "Large" }));
  expect(seen.block.size).toBe("large");
  await userEvent.click(within(screen.getByRole("radiogroup", { name: "Fixed" })).getByRole("radio", { name: "Violet" }));
  expect(seen.block.tone).toBe("violet");
  await userEvent.selectOptions(screen.getByRole("combobox", { name: "Colour" }), "thresholds");
  await userEvent.click(within(screen.getByRole("radiogroup", { name: "Bad when" })).getByRole("radio", { name: "Lower" }));
  expect(seen.block.thresholds).toMatchObject({ direction: "below" });
});

it("a button takes its style and colour from icon choices", async () => {
  const seen = edit({ kind: "button", label: "Stop", action: { refresh: true } });
  await userEvent.click(within(screen.getByRole("radiogroup", { name: "Style" })).getByRole("radio", { name: "Quiet" }));
  expect(seen.block.style).toBe("ghost");
  const colours = screen.getByRole("radiogroup", { name: "Colour" });
  expect(within(colours).getAllByRole("radio")).toHaveLength(16);
  await userEvent.click(within(colours).getByRole("radio", { name: "Red" }));
  expect(seen.block.tone).toBe("red");
});
