import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, it, vi } from "vitest";

import { sessionKey } from "@/entities/session";
import { jsonResponse, renderWithProviders, testQueryClient } from "@/shared/lib/testing";

import { type ButtonLeaf, WidgetButton } from "./widget-button";

const toast = vi.hoisted(() => ({ success: vi.fn(), error: vi.fn() }));
vi.mock("sonner", () => ({ toast }));
const navigation = vi.hoisted(() => ({ push: vi.fn() }));
vi.mock("next/navigation", () => ({ useRouter: () => ({ push: navigation.push, replace: vi.fn() }) }));

const restart: ButtonLeaf = { kind: "button", align: null, valign: null, action: "restart", does: "automation", label: "Restart", icon: "rotate-ccw", style: "primary", tone: null, confirm: "Restart Jellyfin?", link: null };

function signedIn(rights: Record<string, string[]>) {
  const client = testQueryClient();
  client.setQueryDefaults(sessionKey, { staleTime: Infinity });
  client.setQueryData(sessionKey, { name: "anna", group: "family", admin: false, rights });
  return client;
}

afterEach(() => {
  vi.unstubAllGlobals();
  toast.success.mockReset();
  toast.error.mockReset();
  navigation.push.mockReset();
});

it("asks with the button's question, runs the action, and leads to the run", async () => {
  const fetch = vi.fn(async () => jsonResponse({ run_id: "41" }, { status: 202 }));
  vi.stubGlobal("fetch", fetch);
  renderWithProviders(<WidgetButton button={restart} widget="disks" />, signedIn({ automations: ["execute"] }));
  await userEvent.click(screen.getByRole("button", { name: "Restart" }));
  expect(screen.getByText("Restart Jellyfin?")).toBeInTheDocument();
  expect(fetch).not.toHaveBeenCalled();
  await userEvent.click(screen.getAllByRole("button", { name: "Restart" }).at(-1)!);
  await waitFor(() => expect(fetch).toHaveBeenCalledWith("/api/widgets/disks/actions/restart", expect.objectContaining({ method: "POST" })));
  await waitFor(() => expect(toast.success).toHaveBeenCalled());
  const options = toast.success.mock.calls[0][1] as { action: { onClick: () => void } };
  options.action.onClick();
  expect(navigation.push).toHaveBeenCalledWith("/admin/runs/?run=41");
});

it("is disabled with a hint for someone without the right to run what it runs", () => {
  renderWithProviders(<WidgetButton button={restart} widget="disks" />, signedIn({ layout: ["update"] }));
  const button = screen.getByRole("button", { name: "Restart" });
  expect(button).toBeDisabled();
  expect(button).toHaveAttribute("title", "Needs the right to run automations");
});

it("a link opens only a web address, in a new tab", () => {
  const link: ButtonLeaf = { ...restart, does: "link", confirm: null, link: "https://nas.home.lan" };
  const { unmount } = renderWithProviders(<WidgetButton button={link} widget="disks" />, signedIn({}));
  expect(screen.getByRole("link", { name: "Restart" })).toHaveAttribute("target", "_blank");
  unmount();
  renderWithProviders(<WidgetButton button={{ ...link, link: null }} widget="disks" />, signedIn({}));
  expect(screen.getByRole("button", { name: "Restart" })).toBeDisabled();
});

it("a refresh asks the portal again and says the widget is refreshing", async () => {
  vi.stubGlobal("fetch", vi.fn(async () => jsonResponse({ run_id: null }, { status: 202 })));
  const refresh: ButtonLeaf = { ...restart, does: "refresh", confirm: null, label: "Refresh", action: "refresh" };
  renderWithProviders(<WidgetButton button={refresh} widget="disks" />, signedIn({}));
  await userEvent.click(screen.getByRole("button", { name: "Refresh" }));
  await waitFor(() => expect(toast.success).toHaveBeenCalledWith("The widget is refreshing"));
});

it("a toned button takes its colour in each style, and an untoned one keeps the plain style", () => {
  const styles = [
    ["primary", "bg-palette-red"],
    ["secondary", "text-palette-red"],
    ["ghost", "text-palette-red"],
  ] as const;
  for (const [style, colour] of styles) {
    const { unmount } = renderWithProviders(<WidgetButton button={{ ...restart, style, tone: "red", confirm: null }} widget="disks" />, signedIn({ automations: ["execute"] }));
    expect(screen.getByRole("button", { name: "Restart" }), style).toHaveClass(colour);
    unmount();
  }
  renderWithProviders(<WidgetButton button={restart} widget="disks" />, signedIn({ automations: ["execute"] }));
  expect(screen.getByRole("button", { name: "Restart" }).className).not.toMatch(/palette|tone-/);
});
