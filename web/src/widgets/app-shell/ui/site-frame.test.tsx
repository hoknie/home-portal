import { screen, waitFor, within } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";

import { environmentKey } from "@/entities/environment";
import { modulesKey, modulesSchema } from "@/entities/module";
import { sessionKey } from "@/entities/session";
import { apiSamples } from "@/shared/api";
import { jsonResponse, renderWithProviders, testQueryClient } from "@/shared/lib/testing";

import { SiteFrame } from "./site-frame";

const replace = vi.fn();
let pathname = "/";

vi.mock("next/navigation", () => ({
  usePathname: () => pathname,
  useRouter: () => ({ replace, push: vi.fn() }),
}));

afterEach(() => {
  vi.unstubAllGlobals();
  replace.mockReset();
});

function signedIn() {
  const client = testQueryClient();
  client.setQueryData(sessionKey, { name: "admin", group: "admin", admin: true, rights: {} });
  client.setQueryData(environmentKey, { environment: "local", detected: "local", switchable: true, environments: ["local"] });
  client.setQueryDefaults(modulesKey, { staleTime: Infinity });
  client.setQueryData(modulesKey, { data: modulesSchema.parse(apiSamples.modules), revision: '"m"' });
  return client;
}

it("a signed-in person gets the menu on the home page with Home marked", () => {
  pathname = "/";
  renderWithProviders(<SiteFrame guest={<p>guest header</p>}>grid</SiteFrame>, signedIn());
  const menu = screen.getByRole("complementary", { name: "Menu" });
  expect(within(menu).getByRole("link", { name: "Home" })).toHaveAttribute("aria-current", "page");
  expect(within(menu).getByRole("link", { name: "Services" })).not.toHaveAttribute("aria-current");
  expect(screen.queryByText("guest header")).toBeNull();
  expect(screen.getByText("grid")).toBeInTheDocument();
});

it("a service page keeps the menu, collapsed as it was left", () => {
  pathname = "/service/";
  window.localStorage.setItem("home-portal.menu-collapsed", "1");
  renderWithProviders(<SiteFrame guest={<p>guest header</p>}>media</SiteFrame>, signedIn());
  expect(screen.getByRole("complementary", { name: "Menu" }).closest("[data-menu]")).toHaveAttribute("data-menu", "collapsed");
  window.localStorage.clear();
});

it("a guest on the home page sees the guest header, no menu, and is not sent to sign in", async () => {
  pathname = "/";
  vi.stubGlobal("fetch", vi.fn(async () => jsonResponse({ error: "unauthorized" }, { status: 401 })));
  renderWithProviders(<SiteFrame guest={<p>guest header</p>}>grid</SiteFrame>, testQueryClient({ signedIn: false }));
  expect(await screen.findByText("guest header")).toBeInTheDocument();
  expect(screen.queryByRole("complementary", { name: "Menu" })).toBeNull();
  await waitFor(() => expect(screen.getByText("grid")).toBeInTheDocument());
  expect(replace).not.toHaveBeenCalled();
});
