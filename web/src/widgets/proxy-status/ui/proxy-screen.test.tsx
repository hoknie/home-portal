import { screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, it, vi } from "vitest";

import { type Proxy, proxyKey, proxySchema } from "@/entities/proxy";
import { apiSamples } from "@/shared/api";
import { renderWithProviders, testQueryClient } from "@/shared/lib/testing";

import { ProxyScreen } from "./proxy-screen";

function sample(change: (proxy: Proxy) => void = () => undefined) {
  const proxy = proxySchema.parse(structuredClone(apiSamples.proxy));
  change(proxy);
  return proxy;
}

function renderWith(proxy: Proxy) {
  const client = testQueryClient();
  client.setQueryData(proxyKey, { data: proxy, revision: null });
  return renderWithProviders(<ProxyScreen />, client);
}

afterEach(() => {
  vi.unstubAllGlobals();
});

it("lists every published host with its service, certificate and sign-in", () => {
  renderWith(sample());
  expect(screen.getByRole("status")).toHaveTextContent("получил текущую конфигурацию");
  const nas = screen.getByRole("link", { name: "nas.example.com" }).closest("tr") as HTMLElement;
  expect(nas).toHaveTextContent("Let's Encrypt");
  expect(nas).toHaveTextContent("internet");
  expect(within(nas).getByRole("link", { name: "nas" })).toHaveAttribute("href", "/service/?id=nas");
  expect(screen.getByRole("link", { name: "portal.example.com" })).toHaveAttribute("href", "https://portal.example.com");
  expect(screen.getByRole("link", { name: "Скачать сертификат" })).toHaveAttribute("href", "/api/proxy/root-certificate");
});

it("shows caddy as unreachable with the last error while still listing the hosts", () => {
  renderWith(
    sample((proxy) => {
      proxy.reachable = false;
      proxy.in_sync = false;
      proxy.last_error = "Caddy's admin API cannot be reached";
    }),
  );
  expect(screen.getByRole("status")).toHaveTextContent("Недоступен");
  expect(screen.getByText("Caddy's admin API cannot be reached")).toBeInTheDocument();
  expect(screen.getByRole("link", { name: "nas.example.com" })).toBeInTheDocument();
});

it("applying shows the configuration in sync", async () => {
  vi.stubGlobal(
    "fetch",
    vi.fn(async () => new Response(JSON.stringify(apiSamples.proxy), { headers: { "Content-Type": "application/json" } })),
  );
  renderWith(
    sample((proxy) => {
      proxy.reachable = false;
      proxy.in_sync = false;
      proxy.last_applied_at = null;
    }),
  );
  expect(screen.getByText("ещё не применялась")).toBeInTheDocument();
  await userEvent.click(screen.getByRole("button", { name: "Применить сейчас" }));
  expect(await screen.findByText("Работает и получил текущую конфигурацию")).toBeInTheDocument();
  expect(vi.mocked(fetch)).toHaveBeenCalledWith("/api/proxy/apply", expect.objectContaining({ method: "POST" }));
});

it("explains how to enable a disabled proxy and hides the certificate without a local authority", () => {
  renderWith(sample((proxy) => {
    proxy.enabled = false;
    proxy.routes = [];
  }));
  expect(screen.getByRole("note")).toHaveTextContent("Включите его в настройках ниже");
  expect(screen.getByRole("switch", { name: "Прокси включён" })).not.toBeChecked();
  expect(screen.queryByRole("link", { name: "Скачать сертификат" })).not.toBeInTheDocument();
});

it("the hosts card has an inset title and long upstreams wrap inside their cells", () => {
  renderWith(sample());
  const title = screen.getByText("Опубликованные адреса");
  expect(title.closest("[data-slot=card-header]")).not.toBeNull();
  const nas = screen.getByRole("link", { name: "nas.example.com" }).closest("tr") as HTMLElement;
  const upstream = within(nas).getAllByRole("cell")[2].firstElementChild;
  expect(upstream).toHaveClass("whitespace-normal", "break-all");
});
