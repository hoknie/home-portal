"use client";

import type { ReactNode } from "react";

import type { WidgetHeight } from "@/shared/api";
import { cn } from "@/shared/lib/cn";

import { type Position, cellClasses, cellStyle, useAutoRows } from "@/shared/lib/widget-grid";

export type GridCellProps = {
  width: number;
  height: WidgetHeight;
  position?: Position;
  className?: string;
  children: ReactNode;
};

export function GridCell({ width, height, position = null, className, children }: GridCellProps) {
  const auto = height === "auto";
  const [content, rows] = useAutoRows<HTMLDivElement>(auto);
  return (
    <div
      className={cn("@container", cellClasses(position), className)}
      style={cellStyle(width, height, rows, position)}
      data-width={width}
      data-height={height}
      data-column={position?.column}
      data-row={position?.row}
    >
      <div ref={content} className={cn(!auto && "h-full")}>
        {children}
      </div>
    </div>
  );
}
