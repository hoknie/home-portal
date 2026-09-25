import { render, screen } from "@testing-library/react";
import { expect, it } from "vitest";

import { TestIntl } from "@/shared/i18n";

import { StatusBadge, StatusDot } from "./status-badge";

it("names each known state in Russian", () => {
  render(
    <TestIntl>
      <StatusBadge state="up" />
      <StatusBadge state="unreadable" />
    </TestIntl>,
  );
  expect(screen.getByText("Работает")).toHaveAttribute("data-state", "up");
  expect(screen.getByText("Не удалось проверить")).toHaveAttribute("data-state", "unreadable");
});

it("shows a state it does not know as unknown instead of failing", () => {
  render(
    <TestIntl>
      <StatusBadge state="maintenance" />
    </TestIntl>,
  );
  expect(screen.getByText("Неизвестно")).toHaveAttribute("data-state", "unknown");
});

it("colours the dot by state", () => {
  const { container } = render(<StatusDot state="down" />);
  expect(container.querySelector("[data-state=down]")).not.toBeNull();
});
