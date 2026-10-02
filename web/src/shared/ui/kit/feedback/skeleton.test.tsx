import { render } from "@testing-library/react";
import { expect, it } from "vitest";

import { Skeleton } from "./skeleton";

it("renders a pulsing placeholder", () => {
  const { container } = render(<Skeleton className="h-4" />);
  expect(container.firstChild).toHaveClass("animate-pulse");
});
