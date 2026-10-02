import { render, screen } from "@testing-library/react";
import { expect, it } from "vitest";

import { Boundary } from "./boundary";

it("renders its children even where the runtime has no view transitions", () => {
  render(
    <Boundary name="x">
      <p>content</p>
    </Boundary>,
  );
  expect(screen.getByText("content")).toBeInTheDocument();
});
