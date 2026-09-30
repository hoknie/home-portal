import { render, screen } from "@testing-library/react";
import { expect, it } from "vitest";

import { TestIntl } from "@/shared/i18n";

import { FormProblems } from "./form-problems";

it("lists each problem with its path and message", () => {
  render(
    <TestIntl>
      <FormProblems problems={[{ path: "workflows[0].steps[1].url", message: "must be an address" }, { path: "timeout_seconds", message: "validation.automationTimeout" }]} />
    </TestIntl>,
  );
  const alert = screen.getByRole("alert");
  expect(alert).toHaveTextContent("workflows[0].steps[1].url: must be an address");
  expect(alert).toHaveTextContent("timeout_seconds");
  expect(alert).not.toHaveTextContent("validation.automationTimeout");
});

it("shows nothing without problems", () => {
  const { container } = render(
    <TestIntl>
      <FormProblems problems={[]} />
    </TestIntl>,
  );
  expect(container).toBeEmptyDOMElement();
});
