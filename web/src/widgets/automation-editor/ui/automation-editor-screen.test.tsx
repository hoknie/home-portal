import { screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";

import { automationsKey, automationsSchema, catalogueKey, catalogueSchema, scriptsKey, scriptsSchema } from "@/entities/automation";
import { apiSamples } from "@/shared/api";
import { renderWithProviders, testQueryClient } from "@/shared/lib/testing";

import { AutomationEditorScreen } from "./automation-editor-screen";

let search = "";

vi.mock("next/navigation", () => ({
  useSearchParams: () => new URLSearchParams(search),
  usePathname: () => "/admin/automations/edit/",
  useRouter: () => ({ push: vi.fn(), replace: vi.fn() }),
}));

afterEach(() => {
  search = "";
});

function renderScreen(mode: "new" | "edit") {
  const client = testQueryClient();
  for (const key of [automationsKey, catalogueKey, scriptsKey]) {
    client.setQueryDefaults(key, { staleTime: Infinity });
  }
  client.setQueryData(automationsKey, { data: automationsSchema.parse(apiSamples.automations), revision: '"r"' });
  client.setQueryData(catalogueKey, catalogueSchema.parse(apiSamples.automationCatalogue));
  client.setQueryData(scriptsKey, scriptsSchema.parse(apiSamples.automationScripts));
  return renderWithProviders(<AutomationEditorScreen mode={mode} />, client);
}

it("an unknown id says so and links back to the list", () => {
  search = "id=nope";
  renderScreen("edit");
  expect(screen.getByText("Automation not found")).toBeInTheDocument();
  expect(screen.getByRole("link", { name: "Back to automations" }).getAttribute("href")).toMatch(/^\/admin\/automations\/?$/);
});

it("an existing automation opens filled in, with run now beside the title", () => {
  search = "id=restart-media";
  renderScreen("edit");
  expect(screen.getByLabelText("Title")).toHaveValue("Restart Jellyfin when it goes down");
  expect(screen.getByLabelText("Id")).toHaveValue("restart-media");
  expect(screen.getByLabelText("Argument 2")).toHaveValue("{{service.id}}");
  expect(screen.getByRole("button", { name: "Run now" })).toBeEnabled();
});

it("a new automation starts empty", () => {
  renderScreen("new");
  expect(screen.getByLabelText("Title")).toHaveValue("");
  expect(screen.queryByRole("button", { name: "Run now" })).not.toBeInTheDocument();
});
