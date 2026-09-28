import { act, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, it, vi } from "vitest";

import { environmentKey } from "@/entities/environment";
import { type Modules, modulesKey, modulesSchema } from "@/entities/module";
import { sessionKey } from "@/entities/session";
import { apiSamples } from "@/shared/api";
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

function modulesWith(on: string[]): Modules {
  const modules = modulesSchema.parse(structuredClone(apiSamples.modules));
  return { modules: modules.modules.map((module) => ({ ...module, enabled: on.includes(module.name), required_by: [] })) };
}

function shellWith(on: string[] | null) {
  const client = testQueryClient();
  client.setQueryData(sessionKey, { name: "admin" });
  client.setQueryData(environmentKey, { environment: "local", detected: "local", switchable: true, environments: ["local", "vpn", "internet"] });
  if (on) {
    client.setQueryDefaults(modulesKey, { staleTime: Infinity });
    client.setQueryData(modulesKey, { data: modulesWith(on), revision: '"m"' });
  }
  renderWithProviders(<AppShell>content</AppShell>, client);
  return client;
}

it("shows the navigation, marks the current section and names the signed-in user", () => {
  shellWith(["proxy", "automations"]);
  expect(screen.getAllByRole("link", { name: "Back to home" })[0]).toHaveAttribute("href", "/");
  expect(screen.getAllByRole("link", { name: "Services" })[0]).toHaveAttribute("aria-current", "page");
  expect(screen.getAllByRole("link", { name: "Layout" })[0]).toHaveAttribute("href", expect.stringMatching(/^\/admin\/layout\/?$/));
  expect(screen.getAllByRole("link", { name: "Network" }).length).toBeGreaterThan(0);
  expect(screen.getAllByRole("link", { name: "Proxy" })[0]).toHaveAttribute("href", expect.stringMatching(/^\/admin\/proxy\/?$/));
  expect(screen.getAllByText("admin").length).toBeGreaterThan(0);
  expect(screen.getByText("content")).toBeInTheDocument();
  expect(screen.getAllByText("local")[0]).toHaveAttribute("data-environment", "local");
  expect(screen.getAllByRole("button", { name: "Language: English" }).length).toBeGreaterThan(0);
});

it("names the environment the portal placed the visitor in", () => {
  const client = testQueryClient();
  client.setQueryData(sessionKey, { name: "admin" });
  client.setQueryData(environmentKey, { environment: "vpn", environments: ["local", "vpn"] });
  renderWithProviders(<AppShell>content</AppShell>, client);
  expect(screen.getAllByText("vpn")[0]).toHaveAttribute("data-environment", "vpn");
  expect(screen.getAllByText("Environment").length).toBeGreaterThan(0);
});

it("marks a management page seen from another environment and offers going back", () => {
  const client = testQueryClient();
  client.setQueryData(sessionKey, { name: "admin" });
  client.setQueryData(environmentKey, { environment: "internet", detected: "local", switchable: true, environments: ["local", "vpn", "internet"] });
  renderWithProviders(<AppShell>content</AppShell>, client);
  expect(screen.getAllByText("Viewing as from “internet”; you are in “local”").length).toBeGreaterThan(0);
  expect(screen.getAllByRole("button", { name: "Back to “local”" }).length).toBeGreaterThan(0);
});

it("sends a signed-out visitor to the sign-in page, remembering the page", async () => {
  vi.stubGlobal("fetch", vi.fn(async () => new Response("sign in required", { status: 401 })));
  Object.defineProperty(window, "location", { value: { ...window.location, pathname: "/admin/services/", search: "" }, writable: true });
  renderWithProviders(<AppShell>content</AppShell>);
  await waitFor(() => expect(replace).toHaveBeenCalledWith("/login/?next=%2Fadmin%2Fservices%2F"));
  expect(screen.queryByText("content")).not.toBeInTheDocument();
});

it("the user menu offers restarting the portal", async () => {
  const client = testQueryClient();
  client.setQueryData(sessionKey, { name: "admin" });
  client.setQueryData(environmentKey, { environment: "local", environments: ["local"] });
  renderWithProviders(<AppShell>content</AppShell>, client);
  const menus = screen.getAllByRole("button", { name: /admin/ });
  menus[0].focus();
  await userEvent.keyboard("{Enter}");
  expect(await screen.findByRole("menuitem", { name: "Restart portal" })).toBeInTheDocument();
});

it("only enabled modules are listed, in a section of their own", () => {
  shellWith(["proxy", "automations"]);
  const section = screen.getAllByRole("group", { name: "Modules" })[0];
  expect(within(section).getAllByRole("link").map((link) => link.textContent)).toEqual(["Proxy", "Automations"]);
  expect(screen.getAllByRole("link", { name: "Modules" })[0]).toHaveAttribute("href", expect.stringMatching(/^\/admin\/modules\/?$/));
  expect(screen.queryByRole("link", { name: "DNS" })).not.toBeInTheDocument();
});

it("the modules section is hidden when none is on, and while modules load", () => {
  shellWith([]);
  expect(screen.queryByRole("group", { name: "Modules" })).not.toBeInTheDocument();
  expect(screen.getAllByRole("link", { name: "Services" }).length).toBeGreaterThan(0);
});

it("switching a module updates the menu without a reload", async () => {
  const client = shellWith(["automations"]);
  expect(screen.queryByRole("link", { name: "Webhooks" })).not.toBeInTheDocument();
  act(() => {
    client.setQueryData(modulesKey, { data: modulesWith(["automations", "webhooks"]), revision: '"m2"' });
  });
  expect((await screen.findAllByRole("link", { name: "Webhooks" }))[0]).toHaveAttribute("href", expect.stringMatching(/^\/admin\/webhooks\/?$/));
});

it("users appear last in the modules section while their module is on", () => {
  shellWith(["automations", "users"]);
  const section = screen.getAllByRole("group", { name: "Modules" })[0];
  const links = within(section).getAllByRole("link");
  expect(links.map((link) => link.textContent)).toEqual(["Automations", "Users"]);
  expect(links[1]).toHaveAttribute("href", expect.stringMatching(/^\/admin\/users\/?$/));
});

it("the menu collapses into a rail of named icons, is remembered, and expands again", async () => {
  window.localStorage.clear();
  shellWith(["workflows"]);
  const menu = screen.getByRole("complementary", { name: "Menu" });
  await userEvent.click(within(menu).getByRole("button", { name: "Collapse menu" }));
  expect(menu.closest("[data-menu]")).toHaveAttribute("data-menu", "collapsed");
  expect(window.localStorage.getItem("home-portal.menu-collapsed")).toBe("1");
  const workflows = within(menu).getByRole("link", { name: "Workflows" });
  expect(workflows).not.toHaveTextContent("Workflows");
  expect(within(menu).getByRole("link", { name: "Services" })).toHaveAttribute("aria-current", "page");
  await userEvent.hover(workflows);
  expect((await screen.findAllByText("Workflows")).length).toBeGreaterThan(0);
  expect(within(menu).getByRole("button", { name: "admin" })).toBeInTheDocument();
  await userEvent.click(within(menu).getByRole("button", { name: "Expand menu" }));
  expect(menu.closest("[data-menu]")).toHaveAttribute("data-menu", "expanded");
  expect(window.localStorage.getItem("home-portal.menu-collapsed")).toBeNull();
});

it("a collapsed menu stays collapsed after a reload", () => {
  window.localStorage.setItem("home-portal.menu-collapsed", "1");
  shellWith(["workflows"]);
  expect(screen.getByRole("complementary", { name: "Menu" }).closest("[data-menu]")).toHaveAttribute("data-menu", "collapsed");
  window.localStorage.clear();
});
