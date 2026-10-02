import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import { expect, it } from "vitest";

import { ChoiceGroup } from "./choice-group";

function Harness() {
  const [value, setValue] = useState<"card" | "plain">("card");
  return <ChoiceGroup label="Background" value={value} onChange={setValue} choices={[{ value: "card", label: "Card" }, { value: "plain", label: "None" }]} />;
}

it("is a radio group whose choice moves with a click and with the arrows", async () => {
  render(<Harness />);
  expect(screen.getByRole("radio", { name: "Card" })).toHaveAttribute("aria-checked", "true");
  await userEvent.click(screen.getByRole("radio", { name: "None" }));
  expect(screen.getByRole("radio", { name: "None" })).toHaveAttribute("aria-checked", "true");
  await userEvent.keyboard("{ArrowRight}");
  expect(screen.getByRole("radio", { name: "Card" })).toHaveAttribute("aria-checked", "true");
  expect(screen.getByRole("radio", { name: "Card" })).toHaveFocus();
});
