import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, it, vi } from "vitest";

import { renderWithProviders } from "@/shared/lib/testing";

import { Loaded } from "./loaded";
import { SkeletonLines } from "./skeletons";

const query = <Data,>(data: Data | undefined, error: Error | null = null) => ({ data, error, refetch: vi.fn() });

it("shows the skeleton while nothing has arrived", () => {
  renderWithProviders(<Loaded query={query<string>(undefined)} skeleton={<SkeletonLines />}>{(data) => <p>{data}</p>}</Loaded>);
  expect(document.querySelector("[data-skeleton='lines']")).not.toBeNull();
});

it("shows the error with a retry when nothing has arrived", async () => {
  const failed = query<string>(undefined, new Error("timed out"));
  renderWithProviders(<Loaded query={failed} skeleton={<SkeletonLines />}>{(data) => <p>{data}</p>}</Loaded>);
  expect(screen.getByRole("alert")).toHaveTextContent("Could not load the data");
  expect(screen.getByRole("alert")).toHaveTextContent("timed out");
  await userEvent.click(screen.getByRole("button", { name: "Retry" }));
  expect(failed.refetch).toHaveBeenCalledOnce();
});

it("fades the data in", () => {
  renderWithProviders(<Loaded query={query("services")} skeleton={<SkeletonLines />}>{(data) => <p>{data}</p>}</Loaded>);
  expect(screen.getByText("services").closest("[data-slot='appear']")).toHaveClass("fade-in-0");
});

it("keeps the content when a refresh of shown data fails", () => {
  renderWithProviders(<Loaded query={query("services", new Error("offline"))} skeleton={<SkeletonLines />}>{(data) => <p>{data}</p>}</Loaded>);
  expect(screen.getByText("services")).toBeInTheDocument();
  expect(screen.queryByRole("alert")).not.toBeInTheDocument();
});

it("waits for every query, and retries them all", async () => {
  const first = query(1);
  const second = query<string>(undefined, new Error("down"));
  const { rerender } = renderWithProviders(
    <Loaded.all queries={[first, second]} skeleton={<SkeletonLines />}>
      {(count, name) => <p>{`${name} ${count}`}</p>}
    </Loaded.all>,
  );
  await userEvent.click(screen.getByRole("button", { name: "Retry" }));
  expect(first.refetch).toHaveBeenCalledOnce();
  expect(second.refetch).toHaveBeenCalledOnce();
  rerender(
    <Loaded.all queries={[first, query("nas")]} skeleton={<SkeletonLines />}>
      {(count, name) => <p>{`${name} ${count}`}</p>}
    </Loaded.all>,
  );
  expect(screen.getByText("nas 1")).toBeInTheDocument();
});
