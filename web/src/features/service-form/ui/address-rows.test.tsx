import { screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, it, vi } from "vitest";

import { environmentKey, environmentSchema } from "@/entities/environment";
import { proxyKey, proxySchema } from "@/entities/proxy";
import { servicesSchema } from "@/entities/service";
import { apiSamples } from "@/shared/api";
import { jsonResponse, renderWithProviders, testQueryClient } from "@/shared/lib/testing";

import { ServiceForm } from "./service-form";

vi.mock("next/navigation", () => ({ useRouter: () => ({ push: vi.fn(), replace: vi.fn() }) }));

const nas = servicesSchema.parse(apiSamples.services).services[1];

afterEach(() => {
  vi.unstubAllGlobals();
});

function seeded(enabled = true) {
  const client = testQueryClient();
  client.setQueryDefaults(proxyKey, { staleTime: Infinity });
  client.setQueryData(environmentKey, environmentSchema.parse(apiSamples.environment));
  client.setQueryData(proxyKey, { data: { ...proxySchema.parse(apiSamples.proxy), enabled }, revision: null });
  return client;
}

function open(service: typeof nas = nas, client = seeded()) {
  renderWithProviders(
    <ServiceForm service={service} environments={["local", "vpn"]} cancelHref="/service/?id=nas" revision='"r1"' taken={["media", "nas"]} groups={["Media", "Network"]} onSaved={vi.fn()} onConflict={vi.fn()} />,
    client,
  );
}

function sent(fetch: ReturnType<typeof vi.fn>) {
  return JSON.parse(String((fetch.mock.calls[0] as unknown as [string, RequestInit])[1].body));
}

function environmentsOf(row: string) {
  return screen.getByRole("button", { name: `Environments of ${row}` });
}

async function toggle(row: string, environment: string) {
  await userEvent.click(environmentsOf(row));
  await userEvent.click(screen.getByRole("menuitemcheckbox", { name: environment }));
  await userEvent.keyboard("{Escape}");
}

it("publishing a service sends its publication through a row through the proxy", async () => {
  const fetch = vi.fn(async () => jsonResponse(apiSamples.services.services[0]));
  vi.stubGlobal("fetch", fetch);
  open({ ...nas, proxy: null });
  await toggle("Main address", "internet");
  await userEvent.click(screen.getByRole("button", { name: "Add address" }));
  await toggle("Address 2", "internet");
  await userEvent.selectOptions(screen.getByLabelText("Sign-in for Address 2"), "portal");
  await userEvent.type(screen.getByLabelText("Address of Address 2"), "https://Media.Example.com");
  await userEvent.click(screen.getByLabelText("Open Address 2 through the proxy"));
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() => expect(fetch).toHaveBeenCalled());
  const body = sent(fetch);
  expect(body.proxy).toEqual({ host: "media.example.com", environments: ["internet"], auth: ["internet"], tls: null, upstream_verify: true });
  expect(body.url).toBe(nas.url);
  expect(body.environments).toBeNull();
});

it("removing the row through the proxy removes the publication and hides the service there", async () => {
  const fetch = vi.fn(async () => jsonResponse(apiSamples.services.services[1]));
  vi.stubGlobal("fetch", fetch);
  open();
  expect(screen.getByLabelText("Address of Address 2")).toHaveValue("https://nas.example.com");
  await userEvent.click(screen.getByRole("button", { name: "Remove Address 2" }));
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  await waitFor(() => expect(fetch).toHaveBeenCalled());
  expect(sent(fetch)).toMatchObject({ proxy: null, environments: ["local", "vpn"] });
});

it("a disabled proxy is explained without hiding the row through the proxy", () => {
  open(nas, seeded(false));
  expect(screen.getByRole("note")).toHaveTextContent("The proxy is off");
  expect(screen.getByLabelText("Open Address 2 through the proxy")).toBeChecked();
});

it("a main address, a vpn address and a published host are three rows, and no more can be added", () => {
  open({ ...nas, url: "http://192.168.1.50", addresses: { vpn: "http://10.8.0.5" } });
  expect(screen.getAllByRole("listitem").map((item) => item.getAttribute("aria-label"))).toEqual(["Main address", "Address 2", "Address 3"]);
  expect(environmentsOf("Main address")).toHaveTextContent("local");
  expect(screen.getByLabelText("Address of Address 2")).toHaveValue("http://10.8.0.5");
  expect(environmentsOf("Address 2")).toHaveTextContent("vpn");
  expect(screen.getByLabelText("Open Address 3 through the proxy")).toBeChecked();
  expect(screen.getByLabelText("Open Main address through the proxy")).toBeDisabled();
  expect(screen.getByRole("button", { name: "Add address" })).toBeDisabled();
});

it("sign-in on a row without the proxy is refused next to it and nothing is sent", async () => {
  const fetch = vi.fn();
  vi.stubGlobal("fetch", fetch);
  open({ ...nas, url: "http://192.168.1.50", addresses: { vpn: "http://10.8.0.5" } });
  await userEvent.selectOptions(screen.getByLabelText("Sign-in for Address 2"), "portal");
  await userEvent.click(screen.getByRole("button", { name: "Save" }));
  const row = screen.getByRole("listitem", { name: "Address 2" });
  expect(await within(row).findByText(/Only the proxy can ask for sign-in/)).toBeInTheDocument();
  expect(fetch).not.toHaveBeenCalled();
});

it("an environment held by one row cannot be chosen in another", async () => {
  open({ ...nas, proxy: null });
  await userEvent.click(screen.getByRole("button", { name: "Add address" }));
  await userEvent.click(environmentsOf("Address 2"));
  expect(screen.getByRole("menuitemcheckbox", { name: "local" })).toHaveAttribute("aria-disabled", "true");
  expect(screen.getByRole("menuitemcheckbox", { name: "internet" })).toHaveAttribute("aria-disabled", "true");
});
