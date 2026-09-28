import { renderHook } from "@testing-library/react";
import type { ReactNode } from "react";
import { expect, it } from "vitest";

import { TestIntl } from "@/shared/i18n";

import { useTrail } from "./trail";

it("a trail starts at home and names a section by its menu item", () => {
  const { result } = renderHook(() => useTrail(), { wrapper: ({ children }: { children: ReactNode }) => <TestIntl>{children}</TestIntl> });
  expect(result.current.of(result.current.section("workflows"), { label: "Revive" })).toEqual([
    { label: "Home", href: "/" },
    { label: "Workflows", href: "/admin/workflows/" },
    { label: "Revive" },
  ]);
});
