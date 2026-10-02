import { render, screen } from "@testing-library/react";
import { expect, it, vi } from "vitest";

import { PageTransition } from "./page-transition";

vi.mock("next/navigation", () => ({ usePathname: () => "/admin/services/" }));

it("wraps the page content in one transition slot", () => {
  render(
    <PageTransition>
      <p>services</p>
    </PageTransition>,
  );
  expect(screen.getByText("services").closest("[data-slot='page-transition']")).not.toBeNull();
});
