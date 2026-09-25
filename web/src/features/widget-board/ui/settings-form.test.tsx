import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import { expect, it } from "vitest";

import { renderWithProviders } from "@/shared/lib/testing";

import { SettingsForm } from "./settings-form";

function Harness({ type, initial }: { type: string; initial: Record<string, unknown> }) {
  const [value, setValue] = useState(initial);
  return (
    <>
      <SettingsForm type={type} value={value} onChange={setValue} />
      <output>{JSON.stringify(value)}</output>
    </>
  );
}

it("edits weather settings with the provider's rules shown beside the fields", async () => {
  renderWithProviders(<Harness type="weather" initial={{ latitude: 56.95, longitude: 24.11 }} />);
  const latitude = screen.getByLabelText("Широта");
  await userEvent.clear(latitude);
  await userEvent.type(latitude, "120");
  expect(screen.getByText("От −90 до 90")).toBeInTheDocument();
  await userEvent.clear(latitude);
  await userEvent.type(latitude, "59,93");
  expect(screen.queryByText("От −90 до 90")).not.toBeInTheDocument();
  expect(screen.getByRole("status").textContent).toContain('"latitude":59.93');
});

it("edits a list as comma-separated values", async () => {
  renderWithProviders(<Harness type="host-metrics" initial={{}} />);
  await userEvent.type(screen.getByLabelText(/^Точки монтирования/), "/, /data");
  expect(screen.getByRole("status").textContent).toBe('{"disks":["/","/data"]}');
});

it("edits the groups of a services widget as chips, new ones included", async () => {
  renderWithProviders(<Harness type="services" initial={{}} />);
  await userEvent.type(screen.getByLabelText(/^Группы/), "Media{Enter}Network{Enter}");
  expect(screen.getByRole("status").textContent).toBe('{"groups":["Media","Network"]}');
  await userEvent.click(screen.getByRole("button", { name: "Убрать «Media»" }));
  await userEvent.click(screen.getByRole("button", { name: "Убрать «Network»" }));
  expect(screen.getByRole("status").textContent).toBe("{}");
});
