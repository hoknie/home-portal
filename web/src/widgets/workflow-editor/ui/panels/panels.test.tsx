import { cleanup, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, expect, it, vi } from "vitest";

import { dictionaries } from "@/shared/i18n";
import { jsonResponse } from "@/shared/lib/testing";

import { localizedTemplate } from "../../model/localize";
import { inspector, node, openEditor, pressOnCanvas, stubCanvasDom, withSteps } from "../testing-support";
import { LEGEND_KEY } from "./legend";

vi.mock("next/navigation", () => ({ useRouter: () => ({ push: vi.fn(), replace: vi.fn() }) }));

beforeEach(() => {
  stubCanvasDom();
  window.localStorage.clear();
});

afterEach(() => {
  vi.unstubAllGlobals();
});

function english(key: string) {
  return key.split(".").reduce<unknown>((tree, part) => (tree as Record<string, unknown> | undefined)?.[part], dictionaries.en.workflowHelp) as string | undefined;
}

it("an error on a nested field marks its node, lists it and opens its field", async () => {
  vi.stubGlobal("fetch", vi.fn(async () => jsonResponse({ errors: [{ field: "workflows[0].steps[1].then[0].url", message: "must be http or https" }] }, { status: 422 })));
  openEditor(
    withSteps([
      { id: "first", kind: "probe", service: "nas" },
      { id: "check", kind: "if", condition: { left: "{{steps.first.state}}", op: "==", right: "down" }, then: [{ id: "call", kind: "http", url: "http://x" }] },
    ]),
  );
  await userEvent.click(await screen.findByRole("button", { name: "Got it" }));
  await userEvent.click(screen.getByRole("button", { name: /^Save$/ }));
  const panel = await screen.findByRole("list", { name: "Problems" });
  expect(within(await node("call")).getByTitle("1 problem")).toBeInTheDocument();
  await userEvent.click(within(panel).getByRole("button", { name: /must be http or https/ }));
  const url = within(inspector()).getByRole("combobox", { name: "URL" });
  expect(url).toHaveAttribute("aria-invalid", "true");
  await waitFor(() => expect(url).toHaveFocus());
  expect(within(inspector()).getByText("must be http or https")).toBeInTheDocument();
});

it("starting from a template fills the editor with its steps and a translated title", async () => {
  const initial = localizedTemplate("retry", { has: (key) => english(key) !== undefined, text: (key) => english(key) ?? key })!;
  openEditor(null, { initial });
  expect(await node("Try three times")).toBeInTheDocument();
  expect(await node("Ask the address")).toBeInTheDocument();
  expect(await node("Did it answer?")).toBeInTheDocument();
  pressOnCanvas(await node("Start"));
  expect(within(inspector()).getByLabelText("Title")).toHaveValue("Retry a request until it answers");
});

it("the palette explains each kind and filters by typing", async () => {
  openEditor(withSteps([]));
  const slot = await waitFor(() => {
    const found = document.querySelector<HTMLElement>('[data-slot="|steps|0"]');
    expect(found).not.toBeNull();
    return found as HTMLElement;
  });
  pressOnCanvas(slot);
  const dialog = await screen.findByRole("dialog", { name: "Add a step" });
  expect(within(dialog).getByText("Checks a condition and runs one of two branches.")).toBeInTheDocument();
  await userEvent.type(within(dialog).getByRole("textbox", { name: "Find a step" }), "telegr");
  const buttons = within(dialog)
    .getAllByRole("button")
    .filter((button) => button.textContent?.includes("Sends"));
  expect(buttons).toHaveLength(1);
  expect(within(dialog).queryByText("Checks a condition and runs one of two branches.")).toBeNull();
});

it("the legend shows on the first visit and stays dismissed", async () => {
  openEditor(withSteps([{ id: "ping", kind: "http", url: "http://nas.lan" }]));
  const legend = await screen.findByRole("region", { name: "How the editor works" });
  await userEvent.click(within(legend).getByRole("button", { name: "Got it" }));
  expect(window.localStorage.getItem(LEGEND_KEY)).toBe("1");
  cleanup();
  openEditor(withSteps([{ id: "ping", kind: "http", url: "http://nas.lan" }]));
  await node("ping");
  expect(screen.queryByRole("region", { name: "How the editor works" })).toBeNull();
  await userEvent.click(screen.getByRole("button", { name: "How it works" }));
  expect(screen.getByRole("region", { name: "How the editor works" })).toBeInTheDocument();
});
