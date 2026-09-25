import { render, screen } from "@testing-library/react";
import { expect, it } from "vitest";

import { TagList } from "./tag-list";

it("shows each tag and nothing without tags", () => {
  const { container, rerender } = render(<TagList tags={["media", "night"]} />);
  expect(screen.getByText("media")).toBeInTheDocument();
  expect(screen.getByText("night")).toBeInTheDocument();
  rerender(<TagList tags={[]} />);
  expect(container).toBeEmptyDOMElement();
});
