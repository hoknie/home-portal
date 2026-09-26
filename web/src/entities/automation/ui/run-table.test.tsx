import { screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, it, vi } from "vitest";

import { apiSamples } from "@/shared/api";
import { renderWithProviders } from "@/shared/lib/testing";

import { runsSchema } from "../model/schema";
import { RunTable } from "./run-table";

const runs = runsSchema.parse(apiSamples.automationRuns).runs;

it("a running run shows how long it has run and its actions, a finished one only opens", async () => {
  const onOpen = vi.fn();
  renderWithProviders(<RunTable runs={runs} titleOf={(id) => id} onOpen={onOpen} actionsOf={() => <button type="button">stop it</button>} />);
  const [, running, stopped] = screen.getAllByRole("row");
  expect(within(running).getByText("for 12 s")).toBeInTheDocument();
  expect(within(running).getByRole("button", { name: "stop it" })).toBeInTheDocument();
  expect(within(stopped).queryByRole("button", { name: "stop it" })).toBeNull();
  await userEvent.click(within(stopped).getByRole("button", { name: "Open" }));
  expect(onOpen).toHaveBeenCalledWith("4");
});
