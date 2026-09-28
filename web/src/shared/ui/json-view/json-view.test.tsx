import { render, screen } from "@testing-library/react";
import { expect, it } from "vitest";

import { JsonView } from "./json-view";

it("shows a value as a tree whose first level is open and deeper levels folded", () => {
  const { container } = render(<JsonView label="Answer" value={{ state: "up", disks: [{ name: "sda", size: 2 }], ok: true }} />);
  expect(screen.getByRole("list", { name: "Answer" })).toHaveTextContent('state: "up"');
  const levels = [...container.querySelectorAll("details")];
  expect(levels[0]).toHaveAttribute("open");
  expect(levels[1]).not.toHaveAttribute("open");
  expect(levels[1]).toHaveTextContent("disks: [1]");
  expect(screen.getByText("true")).toBeInTheDocument();
});

it("shows a plain value without folding", () => {
  const { container } = render(<JsonView value="just text" />);
  expect(container.querySelector("details")).toBeNull();
  expect(container).toHaveTextContent('"just text"');
});
