import { screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";

import { renderWithProviders, testQueryClient } from "@/shared/lib/testing";

import { useSession } from "./queries";

function Probe() {
  const session = useSession();
  if (session.isPending) {
    return <p>waiting</p>;
  }
  return (
    <>
      <p>guest</p>
      <Child />
    </>
  );
}

function Child() {
  useSession();
  return <p>child</p>;
}

afterEach(() => {
  vi.unstubAllGlobals();
});

it("a refused session is asked once, and parts mounted after the answer do not ask again", async () => {
  const fetch = vi.fn(async () => new Response("sign in required", { status: 401 }));
  vi.stubGlobal("fetch", fetch);
  renderWithProviders(<Probe />, testQueryClient({ signedIn: false }));
  expect(await screen.findByText("child")).toBeInTheDocument();
  await new Promise((resolve) => setTimeout(resolve, 200));
  expect(screen.getByText("guest")).toBeInTheDocument();
  expect(fetch).toHaveBeenCalledTimes(1);
});
