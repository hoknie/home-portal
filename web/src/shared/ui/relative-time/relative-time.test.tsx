import { render, screen } from "@testing-library/react";
import { expect, it, vi } from "vitest";

import { TestIntl } from "@/shared/i18n";

import { RelativeTime } from "./relative-time";

it("says how long ago a moment was", () => {
  vi.setSystemTime(new Date("2026-09-22T10:05:00Z"));
  render(
    <TestIntl>
      <RelativeTime moment="2026-09-22T10:00:00Z" />
    </TestIntl>,
  );
  expect(screen.getByText(/назад/)).toHaveAttribute("dateTime", "2026-09-22T10:00:00Z");
  vi.useRealTimers();
});

it("falls back when there is no moment or it makes no sense", () => {
  render(
    <TestIntl>
      <RelativeTime moment={null} fallback="никогда" />
      <RelativeTime moment="not a date" fallback="никогда" />
    </TestIntl>,
  );
  expect(screen.getAllByText("никогда")).toHaveLength(2);
});

it("says it in the page's language", () => {
  vi.setSystemTime(new Date("2026-09-22T10:02:00Z"));
  render(
    <TestIntl locale="es">
      <RelativeTime moment="2026-09-22T10:00:00Z" />
    </TestIntl>,
  );
  expect(screen.getByText("hace 2 minutos")).toBeInTheDocument();
  vi.useRealTimers();
});
