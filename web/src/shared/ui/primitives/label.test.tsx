import { render, screen } from "@testing-library/react";
import { expect, it } from "vitest";

import { Input } from "./input";
import { Label } from "./label";

it("names the control it is for", () => {
  render(
    <>
      <Label htmlFor="name">name</Label>
      <Input id="name" />
    </>,
  );
  expect(screen.getByLabelText("name")).toHaveAttribute("id", "name");
});
