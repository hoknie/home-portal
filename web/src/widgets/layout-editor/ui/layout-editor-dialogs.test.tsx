import { cleanup, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, expect, it, vi } from "vitest";

import { LAYOUT, mockGeometry, serve } from "./testing";

const navigation = vi.hoisted(() => ({ push: vi.fn() }));

vi.mock("next/navigation", () => ({
  usePathname: () => "/admin/layout/",
  useRouter: () => ({ replace: vi.fn(), push: navigation.push }),
  useSearchParams: () => new URLSearchParams(),
}));

beforeEach(mockGeometry);

afterEach(() => {
  navigation.push.mockReset();
  window.sessionStorage.clear();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

const tileOf = (key: string) => document.querySelector(`[data-widget="${key}"]`) as HTMLElement;

it("settings opened from the editor save the library entry, and the arrangement saves after them with the new revision", async () => {
  const sent = serve();
  const width = await screen.findByRole("slider", { name: "Width of the widget “Weather”" });
  await userEvent.click(within(tileOf("#1")).getByRole("button", { name: "Settings of “Weather”" }));
  const dialog = screen.getByRole("dialog", { name: "Weather" });
  expect(within(dialog).getByRole("region", { name: "Preview" })).toBeInTheDocument();
  const latitude = within(dialog).getByLabelText(/Latitude/);
  await userEvent.clear(latitude);
  await userEvent.type(latitude, "40");
  await userEvent.click(within(dialog).getByRole("tab", { name: "Look" }));
  await userEvent.click(within(within(dialog).getByRole("radiogroup", { name: "Background" })).getByRole("radio", { name: "None" }));
  await userEvent.click(within(dialog).getByRole("button", { name: "Done" }));
  await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  const library = sent.find((request) => request.path === "/api/dashboard/library/riga");
  expect(library).toMatchObject({ method: "PUT", revision: '"r1"' });
  expect(library?.body).toMatchObject({ id: "riga", type: "weather", settings: { latitude: 40 }, appearance: { surface: "plain" } });
  await userEvent.click(width);
  await userEvent.keyboard("{ArrowLeft}");
  await userEvent.click(screen.getByRole("button", { name: "Save layout" }));
  await waitFor(() => expect(sent.some((request) => request.path === "/api/dashboard")).toBe(true));
  expect(sent.find((request) => request.path === "/api/dashboard")?.revision).toBe('"r9"');
});

it("cancelling changed settings asks first and writes nothing", async () => {
  const sent = serve();
  await screen.findByRole("button", { name: "Move “Weather”" });
  await userEvent.click(within(tileOf("#1")).getByRole("button", { name: "Settings of “Weather”" }));
  await userEvent.type(screen.getByLabelText(/^Title/), "Outside");
  await userEvent.keyboard("{Escape}");
  const asking = await screen.findByRole("dialog", { name: "Drop the changes to this widget?" });
  await userEvent.click(within(asking).getByRole("button", { name: "Drop them" }));
  await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  expect(sent).toHaveLength(0);
});

it("chooses the groups of a services widget from the groups the services use", async () => {
  const sent = serve();
  await screen.findByRole("button", { name: "Move “Summary”" });
  const now = document.querySelector('[data-section-block][data-section="now"]') as HTMLElement;
  await userEvent.click(within(now).getByRole("button", { name: "Add widget" }));
  await userEvent.click(within(screen.getByRole("dialog", { name: "Place a widget" })).getByRole("button", { name: /^Home services/ }));
  const placed = document.querySelectorAll('[data-section="now"] [data-widget]');
  await userEvent.click(within(placed[placed.length - 1] as HTMLElement).getByRole("button", { name: "Settings of “Home services”" }));
  const groups = await screen.findByLabelText(/^Groups/);
  await userEvent.click(groups);
  expect(screen.getAllByRole("option").map((option) => option.textContent)).toEqual(["Media", "Network"]);
  await userEvent.click(screen.getByRole("option", { name: "Network" }));
  await userEvent.click(groups);
  await userEvent.click(screen.getByRole("option", { name: "Media" }));
  await userEvent.click(screen.getByRole("button", { name: "Done" }));
  await waitFor(() => expect(sent.some((request) => request.path === "/api/dashboard/library/home-services")).toBe(true));
  expect(sent.find((request) => request.path === "/api/dashboard/library/home-services")?.body).toMatchObject({ settings: { groups: ["Network", "Media"] } });
});

it("a section's title and background are set in a dialog of the same kind", async () => {
  const sent = serve();
  await screen.findByRole("button", { name: "Move “Weather”" });
  await userEvent.click(screen.getAllByRole("button", { name: "Section settings" })[0]);
  const dialog = screen.getByRole("dialog", { name: "Section settings" });
  await userEvent.click(within(within(dialog).getByRole("radiogroup", { name: "Title" })).getByRole("radio", { name: "Hidden" }));
  await userEvent.click(within(within(dialog).getByRole("radiogroup", { name: "Background" })).getByRole("radio", { name: "Card" }));
  await userEvent.click(within(dialog).getByRole("button", { name: "Done" }));
  await userEvent.click(screen.getByRole("button", { name: "Save layout" }));
  await waitFor(() => expect(sent.some((request) => request.path === "/api/dashboard")).toBe(true));
  const saved = sent.find((request) => request.path === "/api/dashboard")?.body as { sections: Array<{ appearance: unknown }> };
  expect(saved.sections[0].appearance).toEqual({ title: "hidden", surface: "card" });
});

it("a custom widget's settings open its builder, and the arrangement is still there on the way back", async () => {
  const placed = { ...LAYOUT, widgets: [...LAYOUT.widgets, { ...LAYOUT.widgets[0], key: "#3", type: "custom", id: "disks", title: "Disks", settings: {}, section: "media", width: 12 }] };
  serve({ layout: placed });
  const width = await screen.findByRole("slider", { name: "Width of the widget “Weather”" });
  await userEvent.click(width);
  await userEvent.keyboard("{ArrowLeft}");
  await userEvent.click(within(tileOf("#3")).getByRole("button", { name: "Settings of “Disks”" }));
  expect(navigation.push).toHaveBeenCalledWith("/admin/layout/widgets/edit/?id=disks&back=layout&place=%233");
  expect(screen.queryByRole("dialog")).toBeNull();
  cleanup();
  vi.unstubAllGlobals();
  const sent = serve({ layout: placed });
  expect(await screen.findByText("There are unsaved changes")).toBeInTheDocument();
  expect(tileOf("#1")).toHaveAttribute("data-width", "3");
  await userEvent.click(screen.getByRole("button", { name: "Save layout" }));
  await waitFor(() => expect(sent.some((request) => request.path === "/api/dashboard")).toBe(true));
  expect(sent.find((request) => request.path === "/api/dashboard")?.body).toMatchObject({ widgets: expect.arrayContaining([expect.objectContaining({ key: "#1", width: 3 })]) });
});
