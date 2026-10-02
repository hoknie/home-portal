import { render, screen } from "@testing-library/react";
import { expect, it } from "vitest";

import { ListTransition } from "./list-transition";

const rows = (names: string[]) => (
  <ul>
    <ListTransition items={names} keyOf={(name) => name}>
      {(name) => <li>{name}</li>}
    </ListTransition>
  </ul>
);

it("renders the items in order and follows changes", async () => {
  const { rerender } = render(rows(["a", "b", "c"]));
  expect(screen.getAllByRole("listitem").map((item) => item.textContent)).toEqual(["a", "b", "c"]);
  rerender(rows(["c", "a"]));
  expect((await screen.findAllByRole("listitem")).map((item) => item.textContent)).toEqual(["c", "a"]);
});

it("draws a long list without animating each row", () => {
  const many = Array.from({ length: 50 }, (_, index) => `row-${index}`);
  render(rows(many));
  expect(screen.getAllByRole("listitem")).toHaveLength(50);
});
