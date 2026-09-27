import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, it, vi } from "vitest";

import { dnsSchema } from "@/entities/dns-server";
import { apiSamples } from "@/shared/api";
import { jsonResponse, renderWithProviders } from "@/shared/lib/testing";

import { DnsSettingsForm, fieldOf } from "./dns-settings-form";

const dns = dnsSchema.parse(apiSamples.dns);

afterEach(() => {
  vi.unstubAllGlobals();
});

it("saves the port with the revision it loaded and keeps the certificate paths", async () => {
  const fetch = vi.fn(async () => jsonResponse(apiSamples.dns));
  vi.stubGlobal("fetch", fetch);
  renderWithProviders(<DnsSettingsForm dns={dns} revision='"r1"' />);
  const port = screen.getByLabelText("Port");
  await userEvent.clear(port);
  await userEvent.type(port, "5353");
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() => expect(fetch).toHaveBeenCalled());
  const [path, init] = fetch.mock.calls[0] as unknown as [string, RequestInit];
  expect(path).toBe("/api/dns");
  expect(init.method).toBe("PUT");
  expect((init.headers as Record<string, string>)["If-Match"]).toBe('"r1"');
  const body = JSON.parse(String(init.body));
  expect(body.port).toBe(5353);
  expect(body.addresses).toEqual({ vpn: ["10.8.0.1"] });
  expect(body.tls).toEqual({ enabled: true, port: 853, certificate: null, key: null });
});

it("shows a server error beside the port", async () => {
  vi.stubGlobal(
    "fetch",
    vi.fn(async () => jsonResponse({ errors: [{ field: "dns.port", message: "must be between 1 and 65535" }] }, { status: 422 })),
  );
  renderWithProviders(<DnsSettingsForm dns={dns} revision='"r1"' />);
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  expect(await screen.findByText("must be between 1 and 65535")).toBeInTheDocument();
});

it("offers one address row per configured environment and maps server fields", () => {
  renderWithProviders(<DnsSettingsForm dns={dns} revision='"r1"' />);
  expect(screen.getByLabelText(/^local/)).toHaveValue("");
  expect(screen.getByLabelText(/^office/)).toHaveValue("");
  expect(screen.getByLabelText(/^vpn/)).toHaveValue("10.8.0.1");
  expect(fieldOf("dns.addresses.vpn")).toBe("addresses.vpn");
  expect(fieldOf("dns.zones[2]")).toBe("zones");
  expect(fieldOf("dns.records[0].name")).toBeNull();
});
