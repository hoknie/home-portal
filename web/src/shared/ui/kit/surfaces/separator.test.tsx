import { render } from "@testing-library/react";
import { expect, it } from "vitest";

import { Separator } from "./separator";

it("draws a decorative line in the chosen orientation", () => {
  const { container } = render(<Separator orientation="vertical" />);
  expect(container.firstChild).toHaveAttribute("data-orientation", "vertical");
});
