import { render, screen } from "@testing-library/react";
import { expect, it } from "vitest";

import { servicesSchema } from "../model/schema";
import { apiSamples } from "@/shared/api";
import { TestIntl } from "@/shared/i18n";

import { ServiceCard } from "./service-card";

const media = servicesSchema.parse(apiSamples.services).services[0];

it("a signed-in card opens the service page and offers the service itself separately", () => {
  render(
    <TestIntl>
      <ServiceCard service={{ ...media, address: "http://192.168.1.10:8096" }} />
    </TestIntl>,
  );
  expect(screen.getByRole("link", { name: /^Media/ }).getAttribute("href")).toMatch(/^\/service\/?\?id=media$/);
  const open = screen.getByRole("link", { name: "Открыть Media" });
  expect(open).toHaveAttribute("href", "http://192.168.1.10:8096");
  expect(open).toHaveAttribute("target", "_blank");
});

it("a public card links straight to the address of the environment the portal answered for", () => {
  render(
    <TestIntl>
      <ServiceCard service={{ ...media, address: "https://media.example.com" }} scope="public" />
    </TestIntl>,
  );
  expect(screen.getByRole("link")).toHaveAttribute("href", "https://media.example.com");
});

it("falls back to the configured url when the portal reported no address", () => {
  render(
    <TestIntl>
      <ServiceCard service={{ ...media, address: "" }} scope="public" />
    </TestIntl>,
  );
  expect(screen.getByRole("link")).toHaveAttribute("href", media.url);
});

it("explains a failure in the status tooltip", () => {
  const nas = servicesSchema.parse(apiSamples.services).services[1];
  const denied = { ...nas, status: { ...nas.status, state: "unreadable" as const, diagnosis: "local-network-denied" as const, last_error: "No route to host" } };
  render(
    <TestIntl>
      <ServiceCard service={denied} />
    </TestIntl>,
  );
  expect(screen.getByTitle("Нет доступа к локальной сети: No route to host")).toBeInTheDocument();
});
