import { screen, within } from "@testing-library/react";
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
  expect(screen.getAllByRole("table")).toHaveLength(2);
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

it("shows one table per group, sorted by name, with services without a group last and no group column", () => {
  const services = structuredClone(apiSamples.services) as { services: Array<{ id: string; name: string; group: string | null }> };
  const [media, nas, printer] = services.services;
  services.services = [
    { ...media, id: "jellyfin", name: "Jellyfin", group: "Media" },
    { ...printer, id: "router", name: "Router", group: "Network" },
    { ...nas, id: "nas", name: "NAS", group: null },
    { ...media, id: "immich", name: "Immich", group: "Media" },
  ];
  renderWith(services);
  const tables = screen.getAllByRole("table");
  expect(tables).toHaveLength(3);
  const names = (table: HTMLElement) => within(table).getAllByRole("row").slice(1).map((row) => within(row).getAllByRole("link")[0].textContent);
  expect(tables.map(names)).toEqual([["Jellyfin", "Immich"], ["Router"], ["NAS"]]);
  expect(screen.getByText("No group")).toBeInTheDocument();
  expect(screen.queryByRole("columnheader", { name: "Group" })).toBeNull();
});
