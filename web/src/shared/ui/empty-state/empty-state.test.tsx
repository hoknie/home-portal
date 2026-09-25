import { render, screen } from "@testing-library/react";
import { Server } from "lucide-react";
import { expect, it } from "vitest";

import { EmptyState } from "./empty-state";

it("explains the emptiness and offers the next step", () => {
  render(<EmptyState icon={Server} title="Nothing yet" description="Add one" action={<button>add</button>} />);
  expect(screen.getByText("Nothing yet")).toBeInTheDocument();
  expect(screen.getByText("Add one")).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "add" })).toBeInTheDocument();
});
