import { screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, expect, it, vi } from "vitest";

import { scriptsKey, scriptsSchema } from "@/entities/automation";
import { modulesKey, modulesSchema } from "@/entities/module";
import { sessionKey } from "@/entities/session";
import { apiSamples } from "@/shared/api";
import { renderWithProviders, testQueryClient } from "@/shared/lib/testing";
import { TooltipProvider } from "@/shared/ui/kit";

import { CATEGORIES_KEY, rereadCollapsedCategories } from "../model/category-state";
import { NavLinks } from "./nav-links";

const current = vi.hoisted(() => ({ path: "/admin/services/" }));

vi.mock("next/navigation", () => ({
  usePathname: () => current.path,
  useRouter: () => ({ replace: vi.fn(), push: vi.fn() }),
}));

beforeEach(() => {
  current.path = "/admin/services/";
  window.localStorage.clear();
  rereadCollapsedCategories();
});

function menu(compact = false) {
  const client = testQueryClient();
  client.setQueryData(sessionKey, { name: "admin", group: "admin", admin: true, rights: {} });
  const modules = modulesSchema.parse(structuredClone(apiSamples.modules));
  client.setQueryDefaults(modulesKey, { staleTime: Infinity });
  client.setQueryData(modulesKey, { data: { modules: modules.modules.map((module) => ({ ...module, enabled: true })) }, revision: '"m"' });
  client.setQueryDefaults(scriptsKey, { staleTime: Infinity });
  client.setQueryData(scriptsKey, { ...scriptsSchema.parse(apiSamples.automationScripts), editing: true });
  return renderWithProviders(
    <TooltipProvider>
      <NavLinks compact={compact} />
    </TooltipProvider>,
    client,
  );
}

const header = (name: string) => screen.getByRole("button", { name });

it("the modules come in categories, each a header that shows or hides its links", async () => {
  menu();
  expect(screen.getAllByRole("button", { expanded: true }).map((button) => button.textContent)).toEqual(["Network", "Automation", "Notifications", "Access"]);
  expect(within(screen.getByRole("group", { name: "Automation" })).getAllByRole("link").map((link) => link.textContent)).toEqual(["Automations", "Webhooks", "Workflows", "Run journal", "Scripts"]);
  await userEvent.click(header("Automation"));
  expect(header("Automation")).toHaveAttribute("aria-expanded", "false");
  expect(screen.queryByRole("link", { name: "Workflows" })).not.toBeInTheDocument();
});

it("a collapsed category survives a reload", async () => {
  const first = menu();
  await userEvent.click(header("Automation"));
  expect(window.localStorage.getItem(CATEGORIES_KEY)).toBe('["automation"]');
  first.unmount();
  rereadCollapsedCategories();
  menu();
  expect(header("Automation")).toHaveAttribute("aria-expanded", "false");
  expect(screen.queryByRole("link", { name: "Webhooks" })).not.toBeInTheDocument();
  expect(header("Network")).toHaveAttribute("aria-expanded", "true");
  expect(screen.getByRole("link", { name: "Proxy" })).toBeInTheDocument();
});

it("a collapsed category that holds the current page marks its header and stays collapsed", () => {
  window.localStorage.setItem(CATEGORIES_KEY, '["automation"]');
  rereadCollapsedCategories();
  current.path = "/admin/workflows/";
  menu();
  expect(header("Automation")).toHaveAttribute("data-active");
  expect(header("Automation")).toHaveAttribute("aria-expanded", "false");
  expect(header("Network")).not.toHaveAttribute("data-active");
});

it("the rail shows every icon, with a divider between categories", () => {
  window.localStorage.setItem(CATEGORIES_KEY, '["network","automation"]');
  rereadCollapsedCategories();
  menu(true);
  for (const name of ["Proxy", "DNS", "Automations", "Workflows", "Scripts", "Users"]) {
    expect(screen.getByRole("link", { name })).toBeInTheDocument();
  }
  expect(screen.queryByRole("button", { name: "Network" })).not.toBeInTheDocument();
  expect(document.querySelectorAll('[data-category] [data-slot="separator"]').length).toBe(4);
});
