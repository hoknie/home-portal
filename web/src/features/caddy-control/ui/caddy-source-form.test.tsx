import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, it, vi } from "vitest";

import { proxySchema } from "@/entities/proxy";
import { apiSamples } from "@/shared/api";
import { jsonResponse, renderWithProviders } from "@/shared/lib/testing";

import { CaddySourceForm } from "./caddy-source-form";

const proxy = proxySchema.parse(structuredClone(apiSamples.proxy));

afterEach(() => {
  vi.unstubAllGlobals();
});

it("pins a version with the revision it loaded", async () => {
  const fetch = vi.fn(async () => jsonResponse({ ...apiSamples.proxy, caddy: { ...apiSamples.proxy.caddy, version: "2.10.2" } }));
  vi.stubGlobal("fetch", fetch);
  renderWithProviders(<CaddySourceForm proxy={proxy} revision='"r1"' />);
  const version = screen.getByLabelText("Версия");
  expect(version).toHaveValue("latest");
  await userEvent.clear(version);
  await userEvent.type(version, "2.10.2");
  await userEvent.click(screen.getByRole("button", { name: "Сохранить источник" }));
  await waitFor(() => expect(fetch).toHaveBeenCalled());
  const [path, init] = fetch.mock.calls[0] as unknown as [string, RequestInit];
  expect(path).toBe("/api/proxy/caddy");
  expect(init.method).toBe("PUT");
  expect((init.headers as Record<string, string>)["If-Match"]).toBe('"r1"');
  expect(JSON.parse(String(init.body))).toEqual({ source: proxy.caddy.source, version: "2.10.2" });
});

it("refuses a plain-http mirror before sending", async () => {
  const fetch = vi.fn();
  vi.stubGlobal("fetch", fetch);
  renderWithProviders(<CaddySourceForm proxy={proxy} revision='"r1"' />);
  const source = screen.getByLabelText("Источник релизов");
  await userEvent.clear(source);
  await userEvent.type(source, "http://mirror.example.com/releases");
  await userEvent.click(screen.getByRole("button", { name: "Сохранить источник" }));
  expect(await screen.findByText(/Нужен адрес https/)).toBeInTheDocument();
  expect(fetch).not.toHaveBeenCalled();
});

it("shows the server's error next to the source field", async () => {
  vi.stubGlobal(
    "fetch",
    vi.fn(async () => jsonResponse({ errors: [{ field: "proxy.caddy.source", message: "the mirror is refused" }] }, { status: 422 })),
  );
  renderWithProviders(<CaddySourceForm proxy={proxy} revision='"r1"' />);
  const source = screen.getByLabelText("Источник релизов");
  await userEvent.clear(source);
  await userEvent.type(source, "https://git.example.com/api/v1/repos/caddy/caddy/releases");
  await userEvent.click(screen.getByRole("button", { name: "Сохранить источник" }));
  expect(await screen.findByText("the mirror is refused")).toBeInTheDocument();
});
