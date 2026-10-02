import { render, screen } from "@testing-library/react";
import { expect, it } from "vitest";

import { Panel } from "./panel";

it("is a labelled section on the panel surface", () => {
  render(<Panel aria-label="settings">body</Panel>);
  const panel = screen.getByRole("region", { name: "settings" });
  expect(panel).toHaveClass("surface-panel", "rounded-2xl", "p-4");
});

it("takes the element the layout needs", () => {
  render(
    <Panel as="aside" padding="none" aria-label="menu">
      links
    </Panel>,
  );
  const aside = screen.getByRole("complementary", { name: "menu" });
  expect(aside.tagName).toBe("ASIDE");
  expect(aside).not.toHaveClass("p-4");
});
