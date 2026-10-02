import type { ComponentType } from "react";
import { afterEach, describe, expect, it, vi } from "vitest";

import { loadingProblems, renderWhileLoading } from "@/shared/lib/testing";


vi.mock("next/navigation", () => ({
  useRouter: () => ({ push: vi.fn(), replace: vi.fn(), back: vi.fn(), refresh: vi.fn(), prefetch: vi.fn() }),
  usePathname: () => "/",
  useSearchParams: () => new URLSearchParams("id=router"),
  useParams: () => ({}),
}));

const PAGES = import.meta.glob("./**/page.tsx", { eager: true }) as Record<string, { default: ComponentType }>;

function routeOf(file: string): string {
  return file.replace(/^\.\//, "/").replace(/\/?page\.tsx$/, "/").replace(/\/\([^)]+\)/g, "");
}

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("every page shows a skeleton while its data loads", () => {
  const routes = Object.entries(PAGES).map(([file, module]) => ({ route: routeOf(file), Page: module.default }));

  it("finds every page", () => {
    expect(routes.length).toBeGreaterThan(25);
  });

  for (const { route, Page } of routes) {
    it(route, async () => {
      const { container, fetched } = await renderWhileLoading(<Page />);
      expect(loadingProblems(container, fetched)).toEqual([]);
    });
  }
});
