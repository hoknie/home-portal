import { screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";

import { environmentKey } from "@/entities/environment";
import { sessionKey } from "@/entities/session";
import { renderWithProviders, testQueryClient } from "@/shared/lib/testing";

import { AppShell } from "./app-shell";

const replace = vi.fn();

vi.mock("next/navigation", () => ({
  usePathname: () => "/admin/services/",
  useRouter: () => ({ replace, push: vi.fn() }),
}));

afterEach(() => {
  vi.unstubAllGlobals();
  replace.mockReset();
});

it("shows the navigation, marks the current section and names the signed-in user", () => {
  const client = testQueryClient();
  client.setQueryData(sessionKey, { name: "admin" });
  client.setQueryData(environmentKey, { environment: "local", detected: "local", switchable: true, environments: ["local", "vpn", "internet"] });
  renderWithProviders(<AppShell>content</AppShell>, client);
  expect(screen.getAllByRole("link", { name: "На главную" })[0]).toHaveAttribute("href", "/");
  expect(screen.getAllByRole("link", { name: "Сервисы" })[0]).toHaveAttribute("aria-current", "page");
  expect(screen.getAllByRole("link", { name: "Раскладка" })[0]).toHaveAttribute("href", expect.stringMatching(/^\/admin\/layout\/?$/));
  expect(screen.getAllByRole("link", { name: "Сеть" }).length).toBeGreaterThan(0);
  expect(screen.getAllByRole("link", { name: "Прокси" })[0]).toHaveAttribute("href", expect.stringMatching(/^\/admin\/proxy\/?$/));
  expect(screen.getAllByText("admin").length).toBeGreaterThan(0);
  expect(screen.getByText("content")).toBeInTheDocument();
  expect(screen.getAllByText("local")[0]).toHaveAttribute("data-environment", "local");
  expect(screen.getAllByRole("button", { name: "Язык: Русский" }).length).toBeGreaterThan(0);
});

it("names the environment the portal placed the visitor in", () => {
  const client = testQueryClient();
  client.setQueryData(sessionKey, { name: "admin" });
  client.setQueryData(environmentKey, { environment: "vpn", environments: ["local", "vpn"] });
  renderWithProviders(<AppShell>content</AppShell>, client);
  expect(screen.getAllByText("vpn")[0]).toHaveAttribute("data-environment", "vpn");
  expect(screen.getAllByText("Окружение").length).toBeGreaterThan(0);
});

it("marks a management page seen from another environment and offers going back", () => {
  const client = testQueryClient();
  client.setQueryData(sessionKey, { name: "admin" });
  client.setQueryData(environmentKey, { environment: "internet", detected: "local", switchable: true, environments: ["local", "vpn", "internet"] });
  renderWithProviders(<AppShell>content</AppShell>, client);
  expect(screen.getAllByText("Вид как из «internet», вы в «local»").length).toBeGreaterThan(0);
  expect(screen.getAllByRole("button", { name: "Вернуть «local»" }).length).toBeGreaterThan(0);
});

it("sends a signed-out visitor to the sign-in page, remembering the page", async () => {
  vi.stubGlobal("fetch", vi.fn(async () => new Response("sign in required", { status: 401 })));
  Object.defineProperty(window, "location", { value: { ...window.location, pathname: "/admin/services/", search: "" }, writable: true });
  renderWithProviders(<AppShell>content</AppShell>);
  await waitFor(() => expect(replace).toHaveBeenCalledWith("/login/?next=%2Fadmin%2Fservices%2F"));
  expect(screen.queryByText("content")).not.toBeInTheDocument();
});
