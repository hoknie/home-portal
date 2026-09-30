import { screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";

import { sessionKey } from "@/entities/session";
import { renderWithProviders, testQueryClient } from "@/shared/lib/testing";

import { AreaGate } from "./area-gate";

vi.mock("next/navigation", () => ({ usePathname: () => "/admin/proxy/" }));

afterEach(() => {
  vi.unstubAllGlobals();
});

function renderAs(rights: Record<string, string[]>) {
  const fetch = vi.fn();
  vi.stubGlobal("fetch", fetch);
  const client = testQueryClient();
  client.setQueryData(sessionKey, { name: "guest", group: null, admin: false, rights });
  renderWithProviders(
    <AreaGate>
      <p>proxy page</p>
    </AreaGate>,
    client,
  );
  return fetch;
}

it("a page opened by its address without read says there is no access and asks for nothing", () => {
  const fetch = renderAs({});
  expect(screen.getByRole("status")).toHaveTextContent("You have no access to this page");
  expect(screen.getByRole("link", { name: "Back to home" })).toHaveAttribute("href", "/");
  expect(screen.queryByText("proxy page")).toBeNull();
  expect(fetch).not.toHaveBeenCalled();
});

it("with read the page is shown", () => {
  renderAs({ proxy: ["read"] });
  expect(screen.getByText("proxy page")).toBeInTheDocument();
});
