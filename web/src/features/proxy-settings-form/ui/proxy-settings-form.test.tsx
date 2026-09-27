import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, it, vi } from "vitest";

import { type Proxy, proxySchema } from "@/entities/proxy";
import { apiSamples } from "@/shared/api";
import { jsonResponse, renderWithProviders } from "@/shared/lib/testing";

import { ProxySettingsForm } from "./proxy-settings-form";

function sample(change: (proxy: Proxy) => void = () => undefined) {
  const proxy = proxySchema.parse(structuredClone(apiSamples.proxy));
  change(proxy);
  return proxy;
}

const disabled = sample((proxy) => {
  proxy.enabled = false;
  proxy.settings.portal_host = null;
  proxy.settings.cookie_domain = null;
});

afterEach(() => {
  vi.unstubAllGlobals();
});

it("saves the portal host with the revision it loaded and sends no switch", async () => {
  const fetch = vi.fn(async () => jsonResponse(apiSamples.proxy));
  vi.stubGlobal("fetch", fetch);
  renderWithProviders(<ProxySettingsForm proxy={disabled} revision='"r1"' />);
  await userEvent.type(screen.getByLabelText("Portal address"), "Portal.Home.Example.com");
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() => expect(fetch).toHaveBeenCalled());
  const [path, init] = fetch.mock.calls[0] as unknown as [string, RequestInit];
  expect(path).toBe("/api/proxy");
  expect(init.method).toBe("PUT");
  expect((init.headers as Record<string, string>)["If-Match"]).toBe('"r1"');
  expect(JSON.parse(String(init.body))).toEqual({
    http_port: 80,
    https_port: 443,
    portal_host: "portal.home.example.com",
    cookie_domain: null,
    tls: { mode: "acme", email: "owner@example.com", certificate: null, key: null },
  });
});

it("the proxy form has no switch", () => {
  renderWithProviders(<ProxySettingsForm proxy={sample()} revision='"r1"' />);
  expect(screen.queryByRole("switch")).not.toBeInTheDocument();
});

it("an empty portal host can be saved while the proxy is off", async () => {
  const fetch = vi.fn(async () => jsonResponse(apiSamples.proxy));
  vi.stubGlobal("fetch", fetch);
  renderWithProviders(<ProxySettingsForm proxy={disabled} revision='"r1"' />);
  await userEvent.clear(screen.getByLabelText(/^HTTPS port/));
  await userEvent.type(screen.getByLabelText(/^HTTPS port/), "8443");
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() => expect(fetch).toHaveBeenCalled());
  expect(JSON.parse(String((fetch.mock.calls[0] as unknown as [string, RequestInit])[1].body)).portal_host).toBeNull();
});

it("shows a refusal about another section as a notice", async () => {
  vi.stubGlobal(
    "fetch",
    vi.fn(async () =>
      jsonResponse({ errors: [{ field: "network.public_url", message: "must be https://<proxy.portal_host>" }] }, { status: 422 }),
    ),
  );
  renderWithProviders(<ProxySettingsForm proxy={sample()} revision='"r1"' />);
  await userEvent.clear(screen.getByLabelText(/^Sign-in domain/));
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  expect(await screen.findByText(/network.public_url: must be https/)).toBeInTheDocument();
});

it("sends other ports and refuses one port for both", async () => {
  const fetch = vi.fn(async () => jsonResponse(apiSamples.proxy));
  vi.stubGlobal("fetch", fetch);
  renderWithProviders(<ProxySettingsForm proxy={sample()} revision='"r1"' />);
  const https = screen.getByLabelText("HTTPS port");
  const http = screen.getByLabelText("HTTP port");
  await userEvent.clear(https);
  await userEvent.type(https, "80");
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  expect(await screen.findByText("The HTTP and HTTPS ports must differ")).toBeInTheDocument();
  expect(fetch).not.toHaveBeenCalled();
  await userEvent.clear(https);
  await userEvent.type(https, "8443");
  await userEvent.clear(http);
  await userEvent.type(http, "8080");
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() => expect(fetch).toHaveBeenCalled());
  const body = JSON.parse(String((fetch.mock.calls[0] as unknown as [string, RequestInit])[1].body));
  expect(body).toMatchObject({ http_port: 8080, https_port: 8443 });
});

