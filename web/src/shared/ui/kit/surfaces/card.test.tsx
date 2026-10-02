import { render, screen } from "@testing-library/react";
import { expect, it } from "vitest";

import { Card, CardContent, CardHeader, CardTitle } from "./card";

it("lays out a titled card", () => {
  render(
    <Card>
      <CardHeader>
        <CardTitle>title</CardTitle>
      </CardHeader>
      <CardContent>body</CardContent>
    </Card>,
  );
  expect(screen.getByText("title")).toHaveAttribute("data-slot", "card-title");
  expect(screen.getByText("body")).toHaveAttribute("data-slot", "card-content");
});

it("is a glass panel", () => {
  render(<Card>glass</Card>);
  expect(screen.getByText("glass")).toHaveClass("glass-panel");
});
