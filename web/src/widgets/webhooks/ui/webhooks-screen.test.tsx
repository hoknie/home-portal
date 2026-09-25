import { screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, it, vi } from "vitest";

import { webhooksKey, webhooksSchema } from "@/entities/webhook";
import { apiSamples } from "@/shared/api";
import { renderWithProviders, testQueryClient } from "@/shared/lib/testing";

import { WebhooksScreen } from "./webhooks-screen";

vi.mock("next/navigation", () => ({ useRouter: () => ({ push: vi.fn(), replace: vi.fn() }) }));

function renderList() {
  const client = testQueryClient();
  client.setQueryDefaults(webhooksKey, { staleTime: Infinity });
  client.setQueryData(webhooksKey, { data: webhooksSchema.parse(apiSamples.webhooks), revision: '"r"' });
  return renderWithProviders(<WebhooksScreen />, client);
}

it("links each webhook to its page, shortens its address and has no variables column", async () => {
  renderList();
  const link = screen.getByRole("link", { name: "Deploy from CI" });
  expect(link.getAttribute("href")).toMatch(/^\/admin\/webhooks\/details\/?\?id=7d3f2a4e-5b1c-4e8f-9a2d-6c0b1e3f4a5d$/);
  expect(screen.getByText("/webhook/7d3f2a4e…4a5d")).toBeInTheDocument();
  expect(screen.queryByRole("columnheader", { name: "Variables" })).not.toBeInTheDocument();
  await userEvent.hover(screen.getByText("/webhook/7d3f2a4e…4a5d"));
  expect((await screen.findAllByText(/\/webhook\/7d3f2a4e-5b1c-4e8f-9a2d-6c0b1e3f4a5d$/)).length).toBeGreaterThan(0);
});

it("filters the webhooks by tag", async () => {
  renderList();
  await userEvent.click(within(screen.getByRole("group", { name: "Tags:" })).getByRole("button", { name: "camera" }));
  expect(screen.queryByText("Deploy from CI")).not.toBeInTheDocument();
  expect(screen.getByText("Motion at the door")).toBeInTheDocument();
});
