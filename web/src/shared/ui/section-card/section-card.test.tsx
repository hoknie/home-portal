import { render, screen } from "@testing-library/react";
import { expect, it } from "vitest";

import { SectionCard } from "./section-card";

it("puts the title, badge and actions in the header and the content below", () => {
  render(
    <SectionCard title="Network" badge={<span>live</span>} actions={<button>edit</button>}>
      body
    </SectionCard>,
  );
  expect(screen.getByText("Network")).toHaveAttribute("data-slot", "card-title");
  expect(screen.getByText("live")).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "edit" }).closest("[data-slot=card-action]")).not.toBeNull();
  expect(screen.getByText("body")).toHaveAttribute("data-slot", "card-content");
});

it("has no header when nothing goes in it", () => {
  const { container } = render(<SectionCard>only body</SectionCard>);
  expect(container.querySelector("[data-slot=card-header]")).toBeNull();
});

it("a flush card has no vertical padding so a table meets its edges", () => {
  const { container } = render(<SectionCard flush>table</SectionCard>);
  expect(container.querySelector("[data-slot=card]")).toHaveClass("py-0");
});

it("a titled flush card keeps its header inset", () => {
  const { container } = render(
    <SectionCard title="Hosts" flush>
      table
    </SectionCard>,
  );
  const card = container.querySelector("[data-slot=card]");
  expect(card).not.toHaveClass("py-0");
  expect(card).toHaveClass("pb-0", "gap-4");
});
