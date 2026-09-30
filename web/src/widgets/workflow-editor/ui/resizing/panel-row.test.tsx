import { fireEvent, screen } from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";

import { NARROW_QUERY } from "../../model/use-editor-state";
import { node, openEditor, openPage, pressOnCanvas, stubCanvasDom, withSteps } from "../testing-support";
import { rereadPanelWidth } from "./use-panel-width";

vi.mock("next/navigation", () => ({ useRouter: () => ({ push: vi.fn(), replace: vi.fn() }) }));

const workflow = withSteps([{ id: "ping", kind: "http", url: "http://nas.lan" }]);

beforeEach(() => {
  stubCanvasDom();
  window.localStorage.clear();
  rereadPanelWidth();
});

afterEach(() => {
  vi.restoreAllMocks();
});

function row() {
  return document.querySelector("[data-panel-row]") as HTMLElement;
}

it("the editor puts the handle between the canvas and the inspector, and End gives the inspector 70% of the row", async () => {
  openEditor(workflow);
  pressOnCanvas(await node("ping"));
  const handle = screen.getByRole("separator", { name: "Width of the side panel" });
  const inspector = screen.getByRole("complementary");
  expect(handle.nextElementSibling).toBe(inspector);
  expect(inspector.className).toContain("md:w-[clamp(20rem,var(--panel-width,26rem),70%)]");
  expect(row().style.getPropertyValue("--panel-width")).toBe("416px");
  fireEvent.keyDown(handle, { key: "End" });
  expect(row().style.getPropertyValue("--panel-width")).toBe("840px");
});

it("the workflow's page shows the same handle beside its history", async () => {
  openPage(workflow, "history");
  const history = await screen.findByRole("complementary", { name: "History" });
  expect(history.previousElementSibling).toHaveAttribute("role", "separator");
});

it("a narrow screen shows no handle", async () => {
  vi.spyOn(window, "matchMedia").mockImplementation((query: string) => ({ matches: query === NARROW_QUERY, media: query, onchange: null, addEventListener: vi.fn(), removeEventListener: vi.fn(), addListener: vi.fn(), removeListener: vi.fn(), dispatchEvent: vi.fn() }));
  openPage(workflow, "history");
  await screen.findByRole("complementary", { name: "History" });
  expect(screen.queryByRole("separator", { name: "Width of the side panel" })).not.toBeInTheDocument();
});
