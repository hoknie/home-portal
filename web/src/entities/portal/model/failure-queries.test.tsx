import { screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";

import { apiSamples } from "@/shared/api";
import { renderWithProviders, testQueryClient } from "@/shared/lib/testing";

import { useFailure } from "./failure-queries";

function Probe() {
  const failure = useFailure();
  const shown = failure.data?.kind === "failed" ? failure.data.report.problems?.length : failure.data?.kind;
  return (
    <>
      <p>{String(shown ?? "nothing")}</p>
      <p>{failure.isError ? "waiting" : "answered"}</p>
    </>
  );
}

afterEach(() => {
  vi.unstubAllGlobals();
});

it("a report from the failure mode is read with its problems", async () => {
  vi.stubGlobal("fetch", vi.fn(async () => new Response(JSON.stringify(apiSamples.failureReport), { status: 200 })));
  renderWithProviders(<Probe />, testQueryClient({ signedIn: false }));
  expect(await screen.findByText("2")).toBeInTheDocument();
});

it("a 404 means the portal runs", async () => {
  vi.stubGlobal("fetch", vi.fn(async () => new Response("not found", { status: 404 })));
  renderWithProviders(<Probe />, testQueryClient({ signedIn: false }));
  expect(await screen.findByText("running")).toBeInTheDocument();
});

it("an unreachable portal keeps the last report and says it is waiting", async () => {
  const fetch = vi
    .fn()
    .mockResolvedValueOnce(new Response(JSON.stringify(apiSamples.failureReport), { status: 200 }))
    .mockRejectedValue(new TypeError("connection refused"));
  vi.stubGlobal("fetch", fetch);
  const client = testQueryClient({ signedIn: false });
  renderWithProviders(<Probe />, client);
  expect(await screen.findByText("2")).toBeInTheDocument();
  await client.refetchQueries({ queryKey: ["portal-failure"] });
  await waitFor(() => expect(screen.getByText("waiting")).toBeInTheDocument());
  expect(screen.getByText("2")).toBeInTheDocument();
});
