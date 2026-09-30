import { screen, waitFor, within } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";

import { type Permissions, permissionsKey, permissionsSchema } from "@/entities/permission";
import { apiSamples } from "@/shared/api";
import { jsonResponse, renderWithProviders, testQueryClient } from "@/shared/lib/testing";

import { PermissionsScreen } from "./permissions-screen";

function sample(change: (permissions: Permissions) => void = () => undefined) {
  const permissions = permissionsSchema.parse(structuredClone(apiSamples.permissions));
  change(permissions);
  return permissions;
}

function renderWith(permissions: Permissions) {
  const client = testQueryClient();
  client.setQueryData(permissionsKey, { data: permissions, revision: null });
  renderWithProviders(<PermissionsScreen />, client);
}

function row(code: string) {
  return document.querySelector(`[data-code="${code}"]`) as HTMLElement;
}

afterEach(() => {
  vi.unstubAllGlobals();
});

it("lists every permission with its state and names the owner", () => {
  renderWith(sample());
  expect(screen.getByText("These permissions belong to /usr/local/bin/home-portal, and every script it runs has them too.")).toBeInTheDocument();
  expect(within(row("local-network")).getByText("Allowed")).toBeInTheDocument();
  expect(within(row("removable-volumes")).getByText("Denied")).toBeInTheDocument();
  expect(within(row("folder:Documents")).getByText("Waiting for an answer")).toBeInTheDocument();
  expect(within(row("folder:Desktop")).getByText("Not needed here")).toBeInTheDocument();
  expect(within(row("automation:System Events")).getByText("Control System Events")).toBeInTheDocument();
  expect(within(row("full-disk-access")).getByText("Unknown")).toBeInTheDocument();
});

it("a denied permission says where to allow it and for whom", () => {
  renderWith(sample());
  expect(
    within(row("removable-volumes")).getByText(
      "Allow /usr/local/bin/home-portal in System Settings › Privacy & Security › Files and Folders.",
    ),
  ).toBeInTheDocument();
  expect(within(row("local-network")).queryByText(/System Settings/)).toBeNull();
});

it("in Russian the states and advice come from the Russian dictionary", () => {
  const client = testQueryClient();
  client.setQueryData(permissionsKey, { data: sample(), revision: null });
  renderWithProviders(<PermissionsScreen />, client, { locale: "ru" });
  expect(within(row("removable-volumes")).getByText("Запрещено")).toBeInTheDocument();
});

it("on another system nothing can be asked for", () => {
  renderWith(
    sample((permissions) => {
      permissions.platform = "linux";
      permissions.permissions = permissions.permissions.map((permission) => ({ ...permission, state: "not-applicable", advice: null }));
    }),
  );
  expect(screen.getByText("These permissions exist only on macOS; on this system nothing needs to be granted.")).toBeInTheDocument();
  expect(screen.queryByRole("button", { name: "Ask again" })).toBeNull();
});

it("while an answer is awaited the list is fetched again until nothing is pending", async () => {
  const answered = sample((permissions) => {
    permissions.permissions = permissions.permissions.map((permission) =>
      permission.state === "pending" ? { ...permission, state: "granted", advice: null } : permission,
    );
  });
  const fetch = vi.fn(async () => jsonResponse(answered));
  vi.stubGlobal("fetch", fetch);
  renderWith(sample());
  await waitFor(() => expect(within(row("folder:Documents")).getByText("Allowed")).toBeInTheDocument(), { timeout: 5_000 });
  const calls = fetch.mock.calls.length;
  await new Promise((resolve) => setTimeout(resolve, 2_500));
  expect(fetch.mock.calls.length).toBe(calls);
}, 10_000);
