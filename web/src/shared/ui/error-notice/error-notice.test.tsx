import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, it, vi } from "vitest";

import { TestIntl } from "@/shared/i18n";

import { ErrorNotice } from "./error-notice";

it("announces the error and retries on request", async () => {
  const retry = vi.fn();
  render(
    <TestIntl>
      <ErrorNotice title="Failed" description="Portal is down" onRetry={retry} />
    </TestIntl>,
  );
  expect(screen.getByRole("alert")).toHaveTextContent("Failed");
  await userEvent.click(screen.getByRole("button", { name: "Retry" }));
  expect(retry).toHaveBeenCalledOnce();
});

it("offers actions of its own below the description", () => {
  render(
    <TestIntl>
      <ErrorNotice title="Conflict" action={<button type="button">Reload</button>} />
    </TestIntl>,
  );
  expect(screen.getByRole("alert")).toContainElement(screen.getByRole("button", { name: "Reload" }));
});
