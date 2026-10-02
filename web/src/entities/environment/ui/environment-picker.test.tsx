import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import { expect, it, vi } from "vitest";

import { renderWithProviders } from "@/shared/lib/testing";

import { EnvironmentPicker } from "./environment-picker";

function Picker({ spy }: { spy: (value: string[] | null) => void }) {
  const [value, setValue] = useState<string[] | null>(null);
  return (
    <EnvironmentPicker
      label="Shown in"
      environments={["local", "vpn", "internet"]}
      value={value}
      current="local"
      onChange={(next) => {
        spy(next);
        setValue(next);
      }}
    />
  );
}

it("chooses environments as tiles, with everywhere when none is chosen and the current one marked", async () => {
  const spy = vi.fn();
  renderWithProviders(<Picker spy={spy} />);
  expect(screen.getByRole("checkbox", { name: /Everywhere/ })).toHaveAttribute("aria-checked", "true");
  expect(screen.getByRole("checkbox", { name: /local/ })).toHaveTextContent("You are here");
  await userEvent.click(screen.getByRole("checkbox", { name: /vpn/ }));
  expect(spy).toHaveBeenLastCalledWith(["vpn"]);
  expect(screen.getByRole("checkbox", { name: /Everywhere/ })).toHaveAttribute("aria-checked", "false");
  await userEvent.click(screen.getByRole("checkbox", { name: /vpn/ }));
  expect(spy).toHaveBeenLastCalledWith(null);
  await userEvent.click(screen.getByRole("checkbox", { name: /internet/ }));
  await userEvent.click(screen.getByRole("checkbox", { name: /Everywhere/ }));
  expect(spy).toHaveBeenLastCalledWith(null);
});
