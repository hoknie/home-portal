import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, it } from "vitest";

import { NativeSelect } from "./native-select";

it("keeps the platform list in the field style", async () => {
  render(
    <NativeSelect aria-label="source" defaultValue="a">
      <option value="a">first</option>
      <option value="b">second</option>
    </NativeSelect>,
  );
  const field = screen.getByLabelText("source");
  await userEvent.selectOptions(field, "b");
  expect(field).toHaveValue("b");
  expect(field).toHaveClass("h-9", "border-input");
});
