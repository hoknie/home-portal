import { screen, within } from "@testing-library/react";
import { expect, it, vi } from "vitest";

import { catalogueKey, catalogueSchema, scriptsKey, scriptsSchema } from "@/entities/automation";
import { webhooksKey, webhooksSchema } from "@/entities/webhook";
import { apiSamples } from "@/shared/api";
import { renderWithProviders, testQueryClient } from "@/shared/lib/testing";

import { WebhookEditorScreen } from "./webhook-editor-screen";

const DEPLOY = "7d3f2a4e-5b1c-4e8f-9a2d-6c0b1e3f4a5d";

vi.mock("next/navigation", () => ({
  useSearchParams: () => new URLSearchParams(`id=${DEPLOY}`),
  usePathname: () => "/admin/webhooks/edit/",
  useRouter: () => ({ push: vi.fn(), replace: vi.fn() }),
}));

it("the webhook editor's trail passes through the webhook's details", () => {
  const client = testQueryClient();
  for (const key of [webhooksKey, catalogueKey, scriptsKey]) {
    client.setQueryDefaults(key, { staleTime: Infinity });
  }
  client.setQueryData(webhooksKey, { data: webhooksSchema.parse(apiSamples.webhooks), revision: '"r"' });
  client.setQueryData(catalogueKey, catalogueSchema.parse(apiSamples.automationCatalogue));
  client.setQueryData(scriptsKey, scriptsSchema.parse(apiSamples.automationScripts));
  renderWithProviders(<WebhookEditorScreen mode="edit" />, client);
  const trail = screen.getByRole("navigation", { name: "Breadcrumbs" });
  expect(within(trail).getAllByRole("listitem").map((item) => item.textContent)).toEqual(["Home…", "Webhooks", "Deploy from CI", "Edit"]);
  expect(within(trail).getByRole("button", { name: "Show the whole path" })).toBeInTheDocument();
  expect(within(trail).getByRole("link", { name: "Deploy from CI" }).getAttribute("href")).toMatch(new RegExp(`^/admin/webhooks/details/?\\?id=${DEPLOY}$`));
  expect(within(trail).getByText("Edit")).toHaveAttribute("aria-current", "page");
});
