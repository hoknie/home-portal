import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, it, vi } from "vitest";

import { apiSamples } from "@/shared/api";
import { jsonResponse, renderWithProviders } from "@/shared/lib/testing";

import { workflowsSchema } from "../model/schema";
import { DeleteWorkflowButton } from "./delete-workflow-button";

const [revive, other] = workflowsSchema.parse(apiSamples.workflows).workflows;

afterEach(() => {
  vi.unstubAllGlobals();
});

it("a workflow in use cannot be deleted and says who uses it", () => {
  renderWithProviders(<DeleteWorkflowButton workflow={revive} revision='"r1"' />);
  const button = screen.getByRole("button", { name: "Delete" });
  expect(button).toBeDisabled();
  expect(button.parentElement).toHaveAttribute("title", expect.stringContaining(revive.used_by[0].title));
});

it("deletes after confirmation with the revision and then reports it", async () => {
  const fetch = vi.fn(async () => jsonResponse({ workflows: [] }));
  vi.stubGlobal("fetch", fetch);
  const onDeleted = vi.fn();
  renderWithProviders(<DeleteWorkflowButton workflow={{ ...other, used_by: [] }} revision='"r1"' labelled onDeleted={onDeleted} />);
  await userEvent.click(screen.getByRole("button", { name: "Delete" }));
  expect(fetch).not.toHaveBeenCalled();
  await userEvent.click(screen.getAllByRole("button", { name: "Delete" }).at(-1)!);
  await waitFor(() => expect(onDeleted).toHaveBeenCalled());
  expect(fetch).toHaveBeenCalledWith(`/api/workflows/${other.id}`, expect.objectContaining({ method: "DELETE", headers: expect.objectContaining({ "If-Match": '"r1"' }) }));
});
