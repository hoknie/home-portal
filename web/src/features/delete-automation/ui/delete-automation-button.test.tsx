import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, it, vi } from "vitest";

import { automationsSchema } from "@/entities/automation";
import { apiSamples } from "@/shared/api";
import { jsonResponse, renderWithProviders } from "@/shared/lib/testing";

import { DeleteAutomationButton } from "./delete-automation-button";

const [restart] = automationsSchema.parse(apiSamples.automations).automations;

afterEach(() => {
  vi.unstubAllGlobals();
});

it("deletes only after confirmation, sending the revision", async () => {
  const fetch = vi.fn(async () => jsonResponse({ automations: [] }));
  vi.stubGlobal("fetch", fetch);
  renderWithProviders(<DeleteAutomationButton automation={restart} revision='"r1"' />);
  await userEvent.click(screen.getByRole("button", { name: "Delete" }));
  expect(screen.getByText("Delete the automation “Restart Jellyfin when it goes down”?")).toBeInTheDocument();
  expect(fetch).not.toHaveBeenCalled();
  await userEvent.click(screen.getAllByRole("button", { name: "Delete" }).at(-1)!);
  await waitFor(() =>
    expect(fetch).toHaveBeenCalledWith(
      "/api/automations/restart-media",
      expect.objectContaining({ method: "DELETE", headers: expect.objectContaining({ "If-Match": '"r1"' }) }),
    ),
  );
});
