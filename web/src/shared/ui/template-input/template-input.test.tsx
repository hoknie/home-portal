import { fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import { describe, expect, it } from "vitest";

import { TestIntl } from "@/shared/i18n";

import { type TemplateRange, type TemplateSuggestion, type Trigger, applied, argumentCompletionAt, completionAt, filterCompletionAt, matching, segmentsOf } from "./completion";
import { TemplateInput } from "./template-input";

const suggestions: TemplateSuggestion[] = [
  { value: "inputs.service", group: "Inputs", description: "The service to check" },
  { value: "loop.item", group: "Loop", description: "The current item", example: "sda" },
  { value: "loop.index", group: "Loop", description: "Its number" },
  { value: "steps.ping.status", group: "Steps", description: "The HTTP status", example: "200" },
];

function Harness({ initial = "", trigger = "braces", problems = [], multiline = false }: { initial?: string; trigger?: Trigger; problems?: TemplateRange[]; multiline?: boolean }) {
  const [value, setValue] = useState(initial);
  return (
    <TestIntl>
      <TemplateInput aria-label="Text" value={value} onChange={setValue} suggestions={suggestions} trigger={trigger} problems={problems} multiline={multiline} />
    </TestIntl>
  );
}

describe("template completion", () => {
  it("finds the open placeholder before the caret and not a closed one", () => {
    expect(completionAt("hi {{lo", 7, "braces")).toEqual({ start: 5, end: 7, query: "lo" });
    expect(completionAt("hi {{loop.item}} x", 18, "braces")).toBeNull();
    expect(completionAt("hi {{a b", 8, "braces")).toBeNull();
    expect(completionAt("nas", 3, "always")).toEqual({ start: 0, end: 3, query: "nas" });
  });

  it("filters by prefix first and inserts with the closing braces once", () => {
    expect(matching(suggestions, "loop").map((item) => item.value)).toEqual(["loop.item", "loop.index"]);
    expect(matching(suggestions, "status").map((item) => item.value)).toEqual(["steps.ping.status"]);
    expect(applied("x {{lo y", { start: 4, end: 6, query: "lo" }, suggestions[1], "braces")).toEqual({ value: "x {{loop.item}} y", caret: 15 });
    expect(applied("x {{lo}} y", { start: 4, end: 6, query: "lo" }, suggestions[1], "braces").value).toBe("x {{loop.item}} y");
  });

  it("cuts the text into plain, template and problem segments", () => {
    const segments = segmentsOf("a {{b.c}} d", [{ start: 2, end: 9, message: "unknown" }]);
    expect(segments.map((segment) => [segment.text, segment.template, segment.problem?.message ?? null])).toEqual([
      ["a ", false, null],
      ["{{b.c}}", true, "unknown"],
      [" d", false, null],
    ]);
  });
});

describe("TemplateInput", () => {
  it("typing {{ opens the grouped suggestions and Enter inserts at the caret", async () => {
    render(<Harness initial="Hello " />);
    const field = screen.getByRole("combobox", { name: "Text" });
    await userEvent.click(field);
    await userEvent.keyboard("{End}{{{{lo");
    const list = screen.getByRole("listbox");
    expect(screen.getByRole("group", { name: "Loop" })).toBeInTheDocument();
    expect(screen.getAllByRole("option").map((option) => option.textContent)).toEqual([expect.stringContaining("loop.item"), expect.stringContaining("loop.index")]);
    expect(field).toHaveAttribute("aria-activedescendant", `${list.id}-0`);
    await userEvent.keyboard("{ArrowDown}");
    expect(field).toHaveAttribute("aria-activedescendant", `${list.id}-1`);
    await userEvent.keyboard("{Enter}");
    expect(field).toHaveValue("Hello {{loop.index}}");
    expect(screen.queryByRole("listbox")).toBeNull();
  });

  it("Tab inserts, Escape closes, and Ctrl+Space opens at the caret", async () => {
    render(<Harness />);
    const field = screen.getByRole("combobox", { name: "Text" });
    await userEvent.click(field);
    await userEvent.keyboard("{{{{st");
    await userEvent.keyboard("{Tab}");
    expect(field).toHaveValue("{{steps.ping.status}}");
    await userEvent.keyboard(" {Control>} {/Control}");
    expect(screen.getByRole("listbox")).toBeInTheDocument();
    expect(field).toHaveValue("{{steps.ping.status}} {{");
    await userEvent.keyboard("{Escape}");
    expect(screen.queryByRole("listbox")).toBeNull();
  });

  it("a click on an option inserts it", async () => {
    render(<Harness multiline />);
    const field = screen.getByRole("combobox", { name: "Text" });
    await userEvent.click(field);
    await userEvent.keyboard("{{{{");
    await userEvent.click(screen.getByRole("option", { name: /inputs\.service/ }));
    expect(field).toHaveValue("{{inputs.service}}");
  });

  it("the always mode suggests plain values from the whole text", () => {
    render(<Harness trigger="always" />);
    const field = screen.getByRole("combobox", { name: "Text" });
    fireEvent.change(field, { target: { value: "ping", selectionStart: 4 } });
    fireEvent.click(field);
    expect(screen.getAllByRole("option")).toHaveLength(1);
  });

  it("templates are highlighted and problems underlined in the mirror", () => {
    const { container } = render(<Harness initial="x {{steps.later.status}}" problems={[{ start: 2, end: 24, message: "no step later comes earlier" }]} />);
    expect(container.querySelector("[data-template]")?.textContent).toBe("{{steps.later.status}}");
    expect(container.querySelector("[data-problem=error]")?.textContent).toBe("{{steps.later.status}}");
  });
});

describe("TemplateInput with values and templates", () => {
  it("a value field switches to template suggestions after {{", async () => {
    function Both() {
      const [value, setValue] = useState("");
      return (
        <TestIntl>
          <TemplateInput
            aria-label="Service"
            trigger="always"
            value={value}
            onChange={setValue}
            suggestions={[{ value: "nas", label: "NAS" }]}
            templateSuggestions={[{ value: "inputs.service", group: "Inputs" }]}
          />
        </TestIntl>
      );
    }
    render(<Both />);
    const field = screen.getByRole("combobox", { name: "Service" });
    await userEvent.click(field);
    await userEvent.keyboard("n");
    expect(screen.getAllByRole("option").map((option) => option.textContent)).toEqual([expect.stringContaining("NAS")]);
    await userEvent.clear(field);
    await userEvent.keyboard("{{{{inp");
    expect(screen.getAllByRole("option").map((option) => option.textContent)).toEqual([expect.stringContaining("inputs.service")]);
    await userEvent.keyboard("{Enter}");
    expect(field).toHaveValue("{{inputs.service}}");
  });
});

describe("filter completion", () => {
  it("after a bar inside an open placeholder it names the value and the filters before", () => {
    expect(filterCompletionAt("{{steps.list.json | pluck('a') | jo", 35)).toEqual({ start: 33, end: 35, query: "jo", subject: "steps.list.json", chain: "pluck('a')" });
    expect(filterCompletionAt("{{steps.list.json |", 19)).toMatchObject({ query: "", subject: "steps.list.json", chain: "" });
    expect(filterCompletionAt("{{steps.list.json}} |", 21)).toBeNull();
    expect(filterCompletionAt("{{a | join(", 11)).toBeNull();
    expect(applied("{{x |}}", { start: 5, end: 5, query: "" }, { value: "length" }, "filters")).toEqual({ value: "{{x | length}}", caret: 12 });
  });

  it("typing a bar offers the filters for the value before it and inserts the chosen one", async () => {
    const offered: string[] = [];
    function Filters() {
      const [value, setValue] = useState("");
      return (
        <TestIntl>
          <TemplateInput
            aria-label="Text"
            value={value}
            onChange={setValue}
            suggestions={suggestions}
            filterSuggestions={(subject) => {
              offered.push(subject);
              return [{ value: "length", label: "length" }, { value: 'join(", ")', label: "join(separator)" }];
            }}
          />
        </TestIntl>
      );
    }
    render(<Filters />);
    const field = screen.getByRole("combobox", { name: "Text" });
    await userEvent.type(field, "{{{{steps.ping.status |");
    expect(screen.getAllByRole("option").map((option) => option.textContent)).toEqual(["length", "join(separator)"]);
    expect(offered.at(-1)).toBe("steps.ping.status");
    await userEvent.keyboard("{ArrowDown}{Enter}");
    expect(field).toHaveValue('{{steps.ping.status | join(", ")');
  });
});

it("inside a filter call, after ( or a comma, the names in scope are completed without quotes", () => {
  const text = "{{steps.x.json | get(lo";
  expect(argumentCompletionAt(text, text.length)).toEqual({ start: text.length - 2, end: text.length, query: "lo" });
  expect(argumentCompletionAt('{{a.b | replace("x", ', 21)).toEqual({ start: 21, end: 21, query: "" });
  expect(argumentCompletionAt('{{a.b | get("lo', 15)).toBeNull();
  expect(argumentCompletionAt("{{a.b | get(x) | up", 19)).toBeNull();
  expect(argumentCompletionAt("{{steps.x", 9)).toBeNull();
});
