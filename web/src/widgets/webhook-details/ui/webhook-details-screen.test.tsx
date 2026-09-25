import { screen } from "@testing-library/react";
import { expect, it, vi } from "vitest";

import { automationsKey, automationsSchema, runsKey, runsSchema } from "@/entities/automation";
import { webhooksKey, webhooksSchema } from "@/entities/webhook";
import { apiSamples } from "@/shared/api";
import { renderWithProviders, testQueryClient } from "@/shared/lib/testing";

import { WebhookDetailsScreen } from "./webhook-details-screen";

const DEPLOY = "7d3f2a4e-5b1c-4e8f-9a2d-6c0b1e3f4a5d";
let search = `id=${DEPLOY}`;

vi.mock("next/navigation", () => ({ useSearchParams: () => new URLSearchParams(search), useRouter: () => ({ push: vi.fn(), replace: vi.fn() }) }));

function renderPage() {
  const client = testQueryClient();
  for (const key of [webhooksKey, automationsKey, runsKey({ webhook: DEPLOY })]) {
    client.setQueryDefaults(key, { staleTime: Infinity });
  }
  client.setQueryData(webhooksKey, { data: webhooksSchema.parse(apiSamples.webhooks), revision: '"r"' });
  client.setQueryData(automationsKey, { data: automationsSchema.parse(apiSamples.automations), revision: '"r"' });
  const many = runsSchema.parse(apiSamples.automationRuns);
  client.setQueryData(runsKey({ webhook: DEPLOY }), { runs: Array.from({ length: 4 }, () => many.runs).flat().map((run, index) => ({ ...run, id: String(index) })) });
  return renderWithProviders(<WebhookDetailsScreen />, client);
}

it("shows the whole webhook and its last ten runs", () => {
  renderPage();
  expect(screen.getByRole("heading", { name: "Deploy from CI" })).toBeInTheDocument();
  expect(screen.getByText(/\/webhook\/7d3f2a4e-5b1c-4e8f-9a2d-6c0b1e3f4a5d$/)).toBeInTheDocument();
  expect(screen.getByText("branch")).toBeInTheDocument();
  expect(screen.getByText("deploy.sh -- {{webhook.branch}}")).toBeInTheDocument();
  expect(screen.getByText("Token required")).toBeInTheDocument();
  expect(screen.getByText("Last 10 runs")).toBeInTheDocument();
  expect(screen.getAllByRole("row")).toHaveLength(11);
});

it("an unknown webhook says so", () => {
  search = "id=nope";
  renderPage();
  search = `id=${DEPLOY}`;
  expect(screen.getByText("Webhook not found")).toBeInTheDocument();
});
