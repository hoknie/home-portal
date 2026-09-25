import { screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";

import { apiSamples } from "@/shared/api";
import { jsonResponse, renderWithProviders } from "@/shared/lib/testing";

import { SiteHeader } from "./site-header";

vi.mock("next/navigation", () => ({ usePathname: () => "/", useRouter: () => ({ replace: vi.fn(), push: vi.fn() }) }));

function serve(signedIn: boolean, portal: unknown = apiSamples.publicPortal, environment: unknown = apiSamples.environment) {
  const fetch = vi.fn(async (input: RequestInfo | URL) => {
    const path = String(input);
    if (path === "/api/session") {
      return signedIn ? jsonResponse({ name: "admin" }) : new Response("sign in required", { status: 401 });
    }
    if (path === "/api/environment") {
      return jsonResponse(environment);
    }
    return jsonResponse(portal);
  });
  vi.stubGlobal("fetch", fetch);
  renderWithProviders(<SiteHeader />);
  return fetch;
}

afterEach(() => vi.unstubAllGlobals());

it("signed out, it names the environment and offers signing in, asking only public endpoints", async () => {
  const fetch = serve(false);
  expect((await screen.findByRole("link", { name: "Войти" })).getAttribute("href")).toMatch(/^\/login\/?$/);
  expect(await screen.findByRole("button", { name: "Окружение: local. Выбрать другое" })).toHaveAttribute("data-environment", "local");
  expect(screen.getByRole("button", { name: "Язык: Русский" })).toBeInTheDocument();
  const asked = fetch.mock.calls.map((call) => String(call[0]));
  expect(asked.filter((path) => !path.startsWith("/api/public/"))).toEqual(["/api/session"]);
});

it("signed in, it offers management and the account menu instead", async () => {
  serve(true);
  expect((await screen.findByRole("link", { name: "Управление" })).getAttribute("href")).toMatch(/^\/admin\/services\/?$/);
  expect(screen.getByText("admin")).toBeInTheDocument();
  expect(screen.queryByRole("link", { name: "Войти" })).not.toBeInTheDocument();
  expect(screen.getByRole("button", { name: "Язык: Русский" })).toBeInTheDocument();
});

it("from outside, it names the environment without offering to change it", async () => {
  serve(false, { ...apiSamples.publicPortal, environment: "internet", detected: "internet", switchable: false, environments: undefined });
  expect(await screen.findByText("internet")).toHaveAttribute("data-environment", "internet");
  expect(screen.queryByRole("button", { name: /Окружение/ })).not.toBeInTheDocument();
});

it("signed in and looking from outside, it says so and offers going back", async () => {
  serve(true, apiSamples.publicPortal, { ...apiSamples.environment, environment: "internet", detected: "local" });
  expect(await screen.findByText("Вид как из «internet», вы в «local»")).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "Вернуть «local»" })).toBeInTheDocument();
});
