import { screen } from "@testing-library/react";
import { expect, it } from "vitest";

import { networkKey, networkSchema } from "@/entities/network";
import { apiSamples } from "@/shared/api";
import { renderWithProviders, testQueryClient } from "@/shared/lib/testing";

import { NetworkScreen } from "./network-screen";

function renderWith(change: (network: ReturnType<typeof networkSchema.parse>) => void = () => undefined) {
  const network = networkSchema.parse(structuredClone(apiSamples.network));
  change(network);
  const client = testQueryClient();
  client.setQueryData(networkKey, { data: network, revision: '"r"' });
  return renderWithProviders(<NetworkScreen />, client);
}

it("shows the address in effect, the restart notice and the interfaces", () => {
  renderWith();
  expect(screen.getByText("127.0.0.1:8080")).toBeInTheDocument();
  expect(screen.getByRole("status")).toHaveTextContent("take effect after the portal restarts");
  expect(screen.getByText("en0")).toBeInTheDocument();
  expect(screen.getByLabelText("Port")).toHaveValue(9090);
});

it("explains an environment override and hides the restart notice when nothing changed", () => {
  renderWith((network) => {
    network.restart_required = false;
    network.effective.overridden = true;
  });
  expect(screen.getByText(/HOME_PORTAL_ADDRESS/)).toBeInTheDocument();
  expect(screen.queryByRole("status")).not.toBeInTheDocument();
});
