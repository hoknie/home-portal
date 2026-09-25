import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, expect, it, vi } from "vitest";

import { clearChoice, readChoice } from "@/entities/environment";
import { renderWithProviders } from "@/shared/lib/testing";

import { EnvironmentSwitch } from "./environment-switch";

const ENVIRONMENTS = ["local", "vpn", "internet"];

afterEach(() => clearChoice());

it("from inside, it lists every environment and looks from the chosen one", async () => {
  const { client } = renderWithProviders(<EnvironmentSwitch environment="local" detected="local" switchable environments={ENVIRONMENTS} />);
  const invalidate = vi.spyOn(client, "invalidateQueries");
  await userEvent.click(screen.getByRole("button", { name: "Окружение: local. Выбрать другое" }));
  expect(screen.getByRole("menuitemradio", { name: "local — ваше" })).toHaveAttribute("aria-checked", "true");
  await userEvent.click(screen.getByRole("menuitemradio", { name: "internet" }));
  expect(readChoice()).toBe("internet");
  expect(invalidate).toHaveBeenCalled();
});

it("says when the view differs from the detected environment and goes back", async () => {
  document.cookie = "portal_environment=internet; Path=/";
  const { client } = renderWithProviders(<EnvironmentSwitch environment="internet" detected="local" switchable environments={ENVIRONMENTS} />);
  const invalidate = vi.spyOn(client, "invalidateQueries");
  expect(screen.getByText("Вид как из «internet», вы в «local»")).toBeInTheDocument();
  await userEvent.click(screen.getByRole("button", { name: "Вернуть «local»" }));
  expect(readChoice()).toBeNull();
  expect(invalidate).toHaveBeenCalled();
});

it("from outside, it only names the environment", () => {
  renderWithProviders(<EnvironmentSwitch environment="internet" detected="internet" switchable={false} environments={[]} />);
  expect(screen.getByText("internet")).toHaveAttribute("data-environment", "internet");
  expect(screen.queryByRole("button")).not.toBeInTheDocument();
});
