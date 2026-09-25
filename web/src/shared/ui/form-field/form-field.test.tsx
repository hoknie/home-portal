import { render, screen } from "@testing-library/react";
import { expect, it } from "vitest";

import { TestIntl } from "@/shared/i18n";

import { FormField } from "./form-field";

it("labels its control and shows the hint when there is no error", () => {
  render(
    <TestIntl>
      <FormField id="port" label="Порт" hint="1–65535" optional>
        <input id="port" />
      </FormField>
    </TestIntl>,
  );
  expect(screen.getByLabelText(/Порт/)).toHaveAttribute("id", "port");
  expect(screen.getByText("1–65535")).toBeInTheDocument();
  expect(screen.getByText("необязательно")).toBeInTheDocument();
});

it("translates an error given as a message key", () => {
  render(
    <TestIntl>
      <FormField id="port" label="Порт" error="validation.port">
        <input id="port" />
      </FormField>
    </TestIntl>,
  );
  expect(screen.getByRole("alert")).toHaveTextContent("От 1 до 65535");
});

it("shows a server message as it is", () => {
  render(
    <TestIntl>
      <FormField id="url" label="Адрес" error="must name a host">
        <input id="url" />
      </FormField>
    </TestIntl>,
  );
  expect(screen.getByRole("alert")).toHaveTextContent("must name a host");
});

it("keeps its rows at the top when a neighbour in the same grid row is taller", () => {
  const { container } = render(
    <TestIntl>
      <FormField id="port" label="Порт">
        <input id="port" />
      </FormField>
    </TestIntl>,
  );
  expect(container.firstChild).toHaveClass("content-start");
});
