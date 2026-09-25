import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, it, vi } from "vitest";

import { servicesSchema } from "@/entities/service";
import { apiSamples } from "@/shared/api";
import { jsonResponse, renderWithProviders } from "@/shared/lib/testing";

import { DeleteServiceButton } from "./delete-service-button";

const nas = servicesSchema.parse(apiSamples.services).services[1];

afterEach(() => {
  vi.unstubAllGlobals();
});

it("deletes only after confirmation, sending the revision", async () => {
  const fetch = vi.fn(async () => jsonResponse({ services: [] }));
  vi.stubGlobal("fetch", fetch);
  renderWithProviders(<DeleteServiceButton service={nas} revision='"r1"' />);
  await userEvent.click(screen.getByRole("button", { name: "Delete" }));
  expect(screen.getByText("Delete the service “NAS”?")).toBeInTheDocument();
  expect(fetch).not.toHaveBeenCalled();
  await userEvent.click(screen.getAllByRole("button", { name: "Delete" }).at(-1)!);
  await waitFor(() => expect(fetch).toHaveBeenCalledWith("/api/services/nas", expect.objectContaining({ method: "DELETE" })));
});
