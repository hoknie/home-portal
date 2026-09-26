import { screen } from "@testing-library/react";
import { expect, it } from "vitest";

import { renderWithProviders } from "@/shared/lib/testing";

import { OutcomeBadge } from "./outcome-badge";

it("names the queued, running and stopped outcomes, and spins while running", () => {
  renderWithProviders(
    <>
      <OutcomeBadge outcome="queued" />
      <OutcomeBadge outcome="running" />
      <OutcomeBadge outcome="stopped" />
    </>,
  );
  expect(screen.getByText("Queued")).toBeInTheDocument();
  expect(screen.getByText("Stopped")).toBeInTheDocument();
  expect(screen.getByText("Running").querySelector(".animate-spin")).not.toBeNull();
});
