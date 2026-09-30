import { screen } from "@testing-library/react";
import { expect, it, vi } from "vitest";

import { renderWithProviders, testQueryClient } from "@/shared/lib/testing";

import { sessionKey } from "../model/queries";
import type { Session } from "../model/schema";
import { RequireRight } from "./require-right";

function renderAs(session: Session) {
  const client = testQueryClient();
  client.setQueryData(sessionKey, session);
  const fetch = vi.fn();
  vi.stubGlobal("fetch", fetch);
  renderWithProviders(
    <RequireRight area="proxy">
      <p>proxy settings</p>
    </RequireRight>,
    client,
  );
  vi.unstubAllGlobals();
  return fetch;
}

it("shows the page with read", () => {
  renderAs({ name: "root", group: "ops", admin: false, rights: { proxy: ["read"] } });
  expect(screen.getByText("proxy settings")).toBeInTheDocument();
});

it("shows the notice without read and mounts nothing of the page", () => {
  const fetch = renderAs({ name: "guest", group: null, admin: false, rights: {} });
  expect(screen.queryByText("proxy settings")).toBeNull();
  expect(screen.getByRole("status")).toHaveTextContent("You have no access to this page");
  expect(fetch).not.toHaveBeenCalled();
});
