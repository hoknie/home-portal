import { render } from "@testing-library/react";
import { expect, it } from "vitest";

import { SkeletonCard, SkeletonForm, SkeletonLines, SkeletonPage, SkeletonTable, SkeletonWidget } from "./skeletons";

it("each preset marks itself and takes the shape it is given", () => {
  const { container } = render(
    <>
      <SkeletonLines lines={4} />
      <SkeletonTable columns={3} rows={2} />
      <SkeletonForm fields={5} />
      <SkeletonCard />
      <SkeletonWidget rows={3} />
      <SkeletonPage />
    </>,
  );
  const kind = (name: string) => container.querySelector(`[data-skeleton='${name}']`) as HTMLElement;
  expect(kind("lines").children).toHaveLength(4);
  expect(kind("table").children).toHaveLength(3);
  expect(kind("table").children[0].children).toHaveLength(3);
  expect(kind("form").children).toHaveLength(5);
  expect(kind("card")).not.toBeNull();
  expect(kind("widget")).toHaveStyle({ minHeight: "272px" });
  expect(kind("page")).toHaveAttribute("aria-busy", "true");
});

it("drops its own panel inside a surface that already has one", () => {
  const { container } = render(<SkeletonTable columns={2} surface={false} />);
  expect(container.querySelector("[data-skeleton='table']")).not.toHaveClass("surface-panel");
});
