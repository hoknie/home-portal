import { useQuery } from "@tanstack/react-query";
import { afterEach, expect, it, vi } from "vitest";

import { loadingProblems, renderWhileLoading } from "./testing";

function Blank() {
  useQuery({ queryKey: ["blank"], queryFn: () => fetch("/api/blank") });
  return null;
}

function Bare() {
  const query = useQuery({ queryKey: ["bare"], queryFn: () => fetch("/api/bare") });
  return <p>{query.isPending ? "…" : "done"}</p>;
}

function Shaped() {
  const query = useQuery({ queryKey: ["shaped"], queryFn: () => fetch("/api/shaped") });
  return query.isPending ? <div data-skeleton="lines" /> : <p>done</p>;
}

afterEach(() => {
  vi.unstubAllGlobals();
});

it("catches a page that renders nothing while it loads", async () => {
  const { container, fetched } = await renderWhileLoading(<Blank />);
  expect(fetched).toBe(true);
  expect(loadingProblems(container, fetched)).toEqual(["renders nothing while its data loads", "waits for data without a skeleton"]);
});

it("catches a page that waits without a skeleton", async () => {
  const { container, fetched } = await renderWhileLoading(<Bare />);
  expect(loadingProblems(container, fetched)).toEqual(["waits for data without a skeleton"]);
});

it("passes a page that shows a skeleton", async () => {
  const { container, fetched } = await renderWhileLoading(<Shaped />);
  expect(loadingProblems(container, fetched)).toEqual([]);
});
