import { render, screen } from "@testing-library/react";
import { expect, it } from "vitest";

import { TestIntl } from "@/shared/i18n";

import { WidgetFrame, WidgetProblem } from "./widget-frame";

it("names the widget and marks data that is no longer current", () => {
  render(
    <TestIntl>
      <WidgetFrame title="Погода" stale problem="upstream is away">
        <p>12°</p>
      </WidgetFrame>
    </TestIntl>,
  );
  expect(screen.getByRole("heading", { level: 2, name: "Погода" })).toBeInTheDocument();
  expect(screen.getByText(/устарели/)).toHaveAttribute("data-stale", "true");
  expect(screen.getByText("12°")).toBeInTheDocument();
});

it("shows a skeleton instead of its content while loading", () => {
  const { container } = render(
    <TestIntl>
      <WidgetFrame title="Погода" loading>
        <p>12°</p>
      </WidgetFrame>
    </TestIntl>,
  );
  expect(container.querySelector("[aria-busy=true]")).not.toBeNull();
  expect(screen.queryByText("12°")).not.toBeInTheDocument();
});

it("explains a failure inside its own frame", () => {
  render(
    <TestIntl>
      <WidgetProblem message="the calendar answered 500" />
    </TestIntl>,
  );
  expect(screen.getByRole("alert")).toHaveTextContent("the calendar answered 500");
});
