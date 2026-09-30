import { screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, it, vi } from "vitest";

import { type Modules, modulesKey, modulesSchema } from "@/entities/module";
import { apiSamples } from "@/shared/api";
import { jsonResponse, renderWithProviders, testQueryClient } from "@/shared/lib/testing";

import { ModulesScreen } from "./modules-screen";

const address = vi.hoisted(() => ({ search: "" }));

vi.mock("next/navigation", () => ({
  useSearchParams: () => new URLSearchParams(address.search),
  usePathname: () => "/admin/modules/",
  useRouter: () => ({ push: vi.fn(), replace: vi.fn() }),
}));

function renderWith(change: (modules: Modules) => void = () => undefined) {
  const modules = modulesSchema.parse(structuredClone(apiSamples.modules));
  change(modules);
  const client = testQueryClient();
  client.setQueryData(modulesKey, { data: modules, revision: '"r"' });
  renderWithProviders(<ModulesScreen />, client);
  return client;
}

function card(name: string) {
  return screen.getByText(name, { selector: "[data-slot=card-title]" }).closest("[data-slot=card]") as HTMLElement;
}

afterEach(() => {
  vi.unstubAllGlobals();
  address.search = "";
  window.history.replaceState(null, "", "/");
});

function shownCards() {
  return [...document.querySelectorAll("[data-slot=card-title]")].map((title) => title.textContent);
}

it("shows every module with its description, requirements and a link to its page", () => {
  renderWith();
  for (const name of ["Proxy", "DNS", "Automations", "Webhooks"]) {
    expect(card(name)).toBeInTheDocument();
  }
  expect(within(card("DNS")).getByText("Needs: Proxy")).toBeInTheDocument();
  expect(within(card("Proxy")).getByText("Needed by: DNS")).toBeInTheDocument();
  expect(within(card("Webhooks")).getByRole("link", { name: "Configure" })).toHaveAttribute("href", "/admin/webhooks");
});

it("switching webhooks on shows it on", async () => {
  const answered = modulesSchema.parse(structuredClone(apiSamples.modules));
  answered.modules[3].enabled = true;
  answered.modules[2].required_by = ["webhooks"];
  vi.stubGlobal("fetch", vi.fn(async () => jsonResponse(answered, { headers: { ETag: '"r2"' } })));
  renderWith();
  await userEvent.click(within(card("Webhooks")).getByRole("switch"));
  await waitFor(() => expect(within(card("Webhooks")).getByRole("switch")).toBeChecked());
  expect(within(card("Automations")).getByText("Needed by: Webhooks")).toBeInTheDocument();
});

it("a locked switch says which module to switch first", () => {
  renderWith((modules) => {
    modules.modules[3].enabled = true;
    modules.modules[2].required_by = ["webhooks"];
  });
  expect(within(card("Automations")).getByRole("switch")).toBeDisabled();
  expect(within(card("Automations")).getByText("Switch Webhooks off first")).toBeInTheDocument();
});

it("incomplete proxy settings name the missing field and link to the proxy page", async () => {
  vi.stubGlobal(
    "fetch",
    vi.fn(async () => jsonResponse({ errors: [{ field: "proxy.portal_host", message: "is required" }] }, { status: 422 })),
  );
  renderWith((modules) => {
    modules.modules[0].enabled = false;
    modules.modules[0].required_by = [];
    modules.modules[1].enabled = false;
  });
  await userEvent.click(within(card("Proxy")).getByRole("switch"));
  const alert = await within(card("Proxy")).findByRole("alert");
  expect(alert).toHaveTextContent("proxy.portal_host");
  expect(within(alert).getByRole("link", { name: "Configure" })).toHaveAttribute("href", "/admin/proxy");
});

it("choosing a category shows only its cards and keeps the choice in the address", async () => {
  renderWith();
  expect(screen.getByRole("button", { name: "All" })).toHaveAttribute("aria-pressed", "true");
  await userEvent.click(screen.getByRole("button", { name: "Network" }));
  expect(shownCards()).toEqual(["Proxy", "DNS"]);
  expect(screen.getByRole("button", { name: "Network" })).toHaveAttribute("aria-pressed", "true");
  expect(`${window.location.pathname}${window.location.search}`).toBe("/admin/modules/?category=network");
  await userEvent.click(screen.getByRole("button", { name: "All" }));
  expect(shownCards()).toHaveLength(7);
  expect(window.location.search).toBe("");
});

it("the address chooses the category on load, and an unknown one shows every card", () => {
  address.search = "category=automation";
  const first = renderWithProviders(<ModulesScreen />, renderClient());
  expect(shownCards()).toEqual(["Automations", "Webhooks", "Workflows"]);
  expect(within(card("Webhooks")).getByText("Automation")).toBeInTheDocument();
  first.unmount();
  address.search = "category=unknown";
  renderWithProviders(<ModulesScreen />, renderClient());
  expect(shownCards()).toHaveLength(7);
  expect(screen.getByRole("button", { name: "All" })).toHaveAttribute("aria-pressed", "true");
});

it("a module can be switched while a category is chosen", async () => {
  const answered = modulesSchema.parse(structuredClone(apiSamples.modules));
  answered.modules[3].enabled = true;
  vi.stubGlobal("fetch", vi.fn(async () => jsonResponse(answered, { headers: { ETag: '"r2"' } })));
  renderWith();
  await userEvent.click(screen.getByRole("button", { name: "Automation" }));
  await userEvent.click(within(card("Webhooks")).getByRole("switch"));
  await waitFor(() => expect(within(card("Webhooks")).getByRole("switch")).toBeChecked());
  expect(shownCards()).toEqual(["Automations", "Webhooks", "Workflows"]);
});

function renderClient() {
  const client = testQueryClient();
  client.setQueryData(modulesKey, { data: modulesSchema.parse(structuredClone(apiSamples.modules)), revision: '"r"' });
  return client;
}
