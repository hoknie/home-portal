import { screen } from "@testing-library/react";
import { expect, it, vi } from "vitest";

import { servicesKey, servicesSchema } from "@/entities/service";
import { apiSamples } from "@/shared/api";
import { renderWithProviders, testQueryClient } from "@/shared/lib/testing";

import { ServicesScreen } from "./services-screen";

vi.mock("next/navigation", () => ({
  useSearchParams: () => new URLSearchParams(""),
  usePathname: () => "/admin/services/",
  useRouter: () => ({ replace: vi.fn(), push: vi.fn() }),
}));

function renderWith(services: unknown) {
  const client = testQueryClient();
  client.setQueryData(servicesKey, { data: servicesSchema.parse(services), revision: '"r"' });
  return renderWithProviders(<ServicesScreen />, client);
}

it("lists every service with its status and links to the page that adds one", () => {
  renderWith(apiSamples.services);
  expect(screen.getAllByRole("row")).toHaveLength(4);
  expect(screen.getByText("Up")).toBeInTheDocument();
  expect(screen.getByText("Down")).toBeInTheDocument();
  expect(screen.getByRole("link", { name: "Add service" }).getAttribute("href")).toMatch(/^\/admin\/services\/new\/?$/);
});

it("links each row to the page that edits it", () => {
  renderWith(apiSamples.services);
  expect(screen.getAllByRole("link", { name: "Edit" })[1].getAttribute("href")).toMatch(/^\/admin\/services\/edit\/?\?id=nas$/);
});

it("invites the first service when there are none", () => {
  renderWith({ services: [] });
  expect(screen.getByText("No services yet")).toBeInTheDocument();
});

it("names why a service is failing and links each name to its page", () => {
  const services = structuredClone(apiSamples.services) as { services: Array<{ status: Record<string, unknown> }> };
  services.services[1].status = { ...services.services[1].status, state: "unreadable", diagnosis: "local-network-denied" };
  renderWith(services);
  expect(screen.getByText("No access to the local network")).toBeInTheDocument();
  expect(screen.getByRole("link", { name: "NAS" }).getAttribute("href")).toMatch(/^\/service\/?\?id=nas$/);
});
