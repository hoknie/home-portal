import { screen } from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";

import { apiSamples } from "@/shared/api";
import { renderWithProviders, testQueryClient } from "@/shared/lib/testing";

import { FatalScreen } from "./fatal-screen";

const replace = vi.fn();

vi.mock("next/navigation", () => ({
  useRouter: () => ({ replace }),
  usePathname: () => "/fatal/",
}));

function answer(status: number, body: unknown) {
  vi.stubGlobal("fetch", vi.fn(async () => new Response(typeof body === "string" ? body : JSON.stringify(body), { status })));
}

beforeEach(() => {
  replace.mockClear();
});

afterEach(() => {
  vi.unstubAllGlobals();
});

it("a local visitor sees every problem with its file and key", async () => {
  answer(200, apiSamples.failureReport);
  renderWithProviders(<FatalScreen />, testQueryClient({ signedIn: false }));
  expect(await screen.findByText("proxy.enabled")).toBeInTheDocument();
  expect(screen.getByText("/Users/ad/.config/home-portal/services.toml")).toBeInTheDocument();
  expect(screen.getByText("is no longer read; switch the module with modules.proxy")).toBeInTheDocument();
  expect(screen.getByText(/starts by itself/)).toBeInTheDocument();
  expect(replace).not.toHaveBeenCalled();
});

it("a visitor from outside sees no file, key or message, only where the log is", async () => {
  answer(200, { ...apiSamples.failureReport, details: false, problems: null });
  renderWithProviders(<FatalScreen />, testQueryClient({ signedIn: false }));
  expect(await screen.findByText("~/Library/Logs/home-portal.log")).toBeInTheDocument();
  expect(screen.getByText("journalctl -u home-portal")).toBeInTheDocument();
  expect(screen.queryByText("proxy.enabled")).not.toBeInTheDocument();
  expect(document.body.textContent).not.toContain("home-portal.toml");
});

it("a running portal sends the visitor home", async () => {
  answer(404, "not found");
  renderWithProviders(<FatalScreen />, testQueryClient({ signedIn: false }));
  expect(await screen.findByText(/Opening the home page/)).toBeInTheDocument();
  expect(replace).toHaveBeenCalledWith("/");
});

it("an unreachable portal is waited for", async () => {
  vi.stubGlobal("fetch", vi.fn(async () => Promise.reject(new TypeError("connection refused"))));
  renderWithProviders(<FatalScreen />, testQueryClient({ signedIn: false }));
  expect(await screen.findByText(/Waiting for the portal/)).toBeInTheDocument();
  expect(replace).not.toHaveBeenCalled();
});
