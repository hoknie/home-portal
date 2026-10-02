import { render, screen } from "@testing-library/react";
import { expect, it } from "vitest";

import { Text } from "./text";

it("uses the scale for size and tone", () => {
  render(
    <>
      <Text>body</Text>
      <Text as="span" size="caption" tone="muted">
        hint
      </Text>
      <Text tone="danger" role="alert">
        error
      </Text>
    </>,
  );
  expect(screen.getByText("body")).toHaveClass("text-sm", "text-foreground");
  expect(screen.getByText("hint").tagName).toBe("SPAN");
  expect(screen.getByText("hint")).toHaveClass("text-xs", "text-muted-foreground");
  expect(screen.getByRole("alert")).toHaveClass("text-destructive");
});
