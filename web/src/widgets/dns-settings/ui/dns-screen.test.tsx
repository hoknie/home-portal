import { screen } from "@testing-library/react";
import { expect, it } from "vitest";

import { dnsKey, dnsSchema } from "@/entities/dns-server";
import { apiSamples } from "@/shared/api";
import { renderWithProviders, testQueryClient } from "@/shared/lib/testing";

import { DnsScreen } from "./dns-screen";

function renderWith(enabled: boolean) {
  const client = testQueryClient();
  client.setQueryDefaults(dnsKey, { staleTime: Infinity });
  client.setQueryData(dnsKey, { data: { ...dnsSchema.parse(apiSamples.dns), enabled }, revision: '"r"' });
  return renderWithProviders(<DnsScreen />, client);
}

it("a dns page while the module is off says so and keeps the settings form", () => {
  renderWith(false);
  expect(screen.getByRole("heading", { name: "Local DNS" })).toBeInTheDocument();
  expect(screen.getAllByRole("status").some((element) => element.textContent?.includes("The DNS module is off"))).toBe(true);
  expect(screen.getByLabelText("Port")).toHaveValue(53);
  expect(screen.getByRole("button", { name: "Save" })).toBeEnabled();
});

it("a dns page while the module is on has no notice and no switch of its own", () => {
  renderWith(true);
  expect(screen.queryByText(/module is off/)).not.toBeInTheDocument();
  expect(screen.queryByRole("switch", { name: "DNS server on" })).not.toBeInTheDocument();
});
