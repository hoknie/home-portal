import { screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";

import { apiSamples } from "@/shared/api";
import { jsonResponse, renderWithProviders } from "@/shared/lib/testing";

import { HomeScreen } from "./home-screen";

vi.mock("next/navigation", () => ({ usePathname: () => "/", useRouter: () => ({ replace: vi.fn(), push: vi.fn() }) }));

type Answers = Record<string, unknown>;

function serve(answers: Answers, signedIn: boolean) {
  const fetch = vi.fn(async (input: RequestInfo | URL) => {
    const path = String(input);
    if (path === "/api/session") {
      return signedIn ? jsonResponse({ name: "admin" }) : new Response("sign in required", { status: 401 });
    }
    if (path.includes("/widgets/")) {
      return jsonResponse(apiSamples.widgetWeather);
    }
    const found = Object.entries(answers).find(([prefix]) => path.startsWith(prefix));
    return found ? jsonResponse(found[1], { headers: { ETag: '"r"' } }) : new Response("missing", { status: 404 });
  });
  vi.stubGlobal("fetch", fetch);
  renderWithProviders(<HomeScreen />);
  return fetch;
}

afterEach(() => vi.unstubAllGlobals());

it("signed out, it shows what is public in its sections, asking the public half only", async () => {
  const fetch = serve({ "/api/public/portal": apiSamples.publicPortal }, false);
  expect(await screen.findByText("12°C")).toBeInTheDocument();
  expect(screen.getByRole("heading", { level: 2, name: "Now" })).toBeInTheDocument();
  expect(screen.getByRole("link", { name: /Media/ })).toHaveAttribute("href", "http://192.168.1.10:8096");
  const asked = fetch.mock.calls.map((call) => String(call[0]));
  expect(asked.filter((path) => !path.startsWith("/api/public/"))).toEqual(["/api/session"]);
});

it("signed out with nothing public, it says so", async () => {
  serve({ "/api/public/portal": { environment: "internet", detected: "internet", switchable: false, sections: [], services: [], widgets: [] } }, false);
  expect(await screen.findByText("No open services")).toBeInTheDocument();
});

it("signed in, the same screen shows every section, without the management sidebar", async () => {
  serve(
    {
      "/api/dashboard": apiSamples.dashboard,
      "/api/services": apiSamples.services,
      "/api/environment": apiSamples.environment,
    },
    true,
  );
  const mediaHeadings = await screen.findAllByRole("heading", { level: 2, name: "Media" });
  expect(mediaHeadings.some((heading) => heading.parentElement?.getAttribute("data-section") === "media")).toBe(true);
  expect(screen.getByRole("heading", { level: 2, name: "Now" })).toBeInTheDocument();
  expect(screen.queryByRole("link", { name: "Back to home" })).not.toBeInTheDocument();
  expect(screen.getAllByRole("link", { name: "Open Media" }).length).toBeGreaterThan(0);
});

it("signed in with an empty layout, it leads to the layout editor", async () => {
  serve(
    {
      "/api/dashboard": { sections: [{ id: "main", title: null }], widgets: [] },
      "/api/services": apiSamples.services,
      "/api/environment": apiSamples.environment,
    },
    true,
  );
  const link = await screen.findByRole("link", { name: "Open the layout editor" });
  expect(link.getAttribute("href")).toMatch(/^\/admin\/layout\/?$/);
});
