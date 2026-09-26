import { screen, within } from "@testing-library/react";
import { expect, it } from "vitest";

import { dnsKey, dnsSchema } from "@/entities/dns-server";
import { apiSamples } from "@/shared/api";
import { renderWithProviders, testQueryClient } from "@/shared/lib/testing";

import { DnsCard } from "./dns-card";

function renderCard() {
  const client = testQueryClient();
  client.setQueryDefaults(dnsKey, { staleTime: Infinity });
  client.setQueryData(dnsKey, { data: dnsSchema.parse(apiSamples.dns), revision: '"r"' });
  return renderWithProviders(<DnsCard />, client);
}

it("lists each name with its answer in each environment", () => {
  renderCard();
  const row = screen.getByText("jellyfin.home").closest("tr")!;
  const cells = within(row).getAllByRole("cell");
  expect(cells.map((cell) => cell.textContent)).toEqual(["jellyfin.home", "192.168.1.60", "—", "10.8.0.1"]);
});

it("shows a port that cannot be bound as not listening with the reason", () => {
  renderCard();
  const plain = screen.getAllByRole("status").find((element) => element.dataset.transport === "plain")!;
  expect(plain).toHaveTextContent("Not listening");
  expect(plain).toHaveTextContent("Address already in use");
  expect(screen.getByLabelText("Port")).toHaveValue(53);
});

it("warns about an environment without an address and offers the addresses for phones", () => {
  renderCard();
  expect(screen.getAllByRole("alert").some((alert) => alert.textContent?.includes("“office” has no address"))).toBe(true);
  expect(screen.getByText("portal.home", { selector: "code" })).toBeInTheDocument();
  expect(screen.getByText("https://portal.home/dns-query")).toBeInTheDocument();
});
