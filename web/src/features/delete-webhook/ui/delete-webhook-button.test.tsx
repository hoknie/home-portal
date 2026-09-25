import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, it, vi } from "vitest";

import { webhooksSchema } from "@/entities/webhook";
import { apiSamples } from "@/shared/api";
import { jsonResponse, renderWithProviders } from "@/shared/lib/testing";

import { DeleteWebhookButton } from "./delete-webhook-button";

const [deploy] = webhooksSchema.parse(apiSamples.webhooks).webhooks;

afterEach(() => {
  vi.unstubAllGlobals();
});

it("deletes only after confirmation, sending the revision", async () => {
  const fetch = vi.fn(async () => jsonResponse({ webhooks: [] }));
  vi.stubGlobal("fetch", fetch);
  renderWithProviders(<DeleteWebhookButton webhook={deploy} revision='"r1"' />);
  await userEvent.click(screen.getByRole("button", { name: "Delete" }));
  expect(fetch).not.toHaveBeenCalled();
  await userEvent.click(screen.getAllByRole("button", { name: "Delete" }).at(-1)!);
  await waitFor(() => expect(fetch).toHaveBeenCalledWith(`/api/webhooks/${deploy.id}`, expect.objectContaining({ method: "DELETE" })));
});
