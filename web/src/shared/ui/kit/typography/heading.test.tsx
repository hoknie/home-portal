import { render, screen } from "@testing-library/react";
import { expect, it } from "vitest";

import { Heading } from "./heading";

it("maps each level to its element and size", () => {
  render(
    <>
      <Heading level="page">page</Heading>
      <Heading level="section">section</Heading>
      <Heading level="group">group</Heading>
    </>,
  );
  expect(screen.getByRole("heading", { level: 1, name: "page" })).toHaveClass("text-2xl");
  expect(screen.getByRole("heading", { level: 2, name: "section" })).toHaveClass("text-lg");
  expect(screen.getByRole("heading", { level: 3, name: "group" })).toHaveClass("text-sm");
});

it("keeps the document outline when a level's look is needed at another depth", () => {
  render(
    <Heading level="group" as="h2">
      now
    </Heading>,
  );
  expect(screen.getByRole("heading", { level: 2, name: "now" })).toHaveClass("text-sm");
});
