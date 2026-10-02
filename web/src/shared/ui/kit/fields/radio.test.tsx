import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, it } from "vitest";

import { Radio } from "./radio";

it("is one choice of a named group", async () => {
  render(
    <>
      <Radio name="action" value="run" aria-label="run" defaultChecked />
      <Radio name="action" value="workflow" aria-label="workflow" />
    </>,
  );
  await userEvent.click(screen.getByRole("radio", { name: "workflow" }));
  expect(screen.getByRole("radio", { name: "workflow" })).toBeChecked();
  expect(screen.getByRole("radio", { name: "run" })).not.toBeChecked();
});
