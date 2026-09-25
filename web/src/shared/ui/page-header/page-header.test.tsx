import { render, screen } from "@testing-library/react";
import { expect, it } from "vitest";

import { PageHeader } from "./page-header";

it("shows a heading, its description and actions", () => {
  render(<PageHeader title="Services" description="All of them" actions={<button>add</button>} />);
  expect(screen.getByRole("heading", { level: 1, name: "Services" })).toBeInTheDocument();
  expect(screen.getByText("All of them")).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "add" })).toBeInTheDocument();
});
