import { cn } from "@/shared/lib/cn";

import { Skeleton } from "./skeleton";

const ROW_PIXELS = 80;
const GAP_PIXELS = 16;

export type SkeletonLinesProps = { lines?: number; className?: string };

export function SkeletonLines({ lines = 3, className }: SkeletonLinesProps) {
  return (
    <div data-skeleton="lines" aria-busy="true" className={cn("grid gap-2", className)}>
      {Array.from({ length: lines }, (_, index) => (
        <Skeleton key={index} className={cn("h-4", index === lines - 1 ? "w-2/3" : "w-full")} />
      ))}
    </div>
  );
}

export type SkeletonTableProps = { columns: number; rows?: number; surface?: boolean; className?: string };

export function SkeletonTable({ columns, rows = 5, surface = true, className }: SkeletonTableProps) {
  const grid = { gridTemplateColumns: `repeat(${columns}, minmax(0, 1fr))` };
  return (
    <div data-skeleton="table" aria-busy="true" className={cn("grid gap-3 p-4", surface && "surface-panel rounded-2xl", className)}>
      <div className="grid gap-4" style={grid}>
        {Array.from({ length: columns }, (_, column) => (
          <Skeleton key={column} className="h-3 w-1/2" />
        ))}
      </div>
      {Array.from({ length: rows }, (_, row) => (
        <div key={row} className="grid gap-4 border-t border-glass-edge pt-3" style={grid}>
          {Array.from({ length: columns }, (_, column) => (
            <Skeleton key={column} className={cn("h-4", column === 0 ? "w-3/4" : "w-1/2")} />
          ))}
        </div>
      ))}
    </div>
  );
}

export type SkeletonFormProps = { fields?: number; surface?: boolean; className?: string };

export function SkeletonForm({ fields = 4, surface = true, className }: SkeletonFormProps) {
  return (
    <div data-skeleton="form" aria-busy="true" className={cn("grid gap-5 p-4 sm:p-5", surface && "surface-panel rounded-2xl", className)}>
      {Array.from({ length: fields }, (_, index) => (
        <div key={index} className="grid gap-2">
          <Skeleton className="h-3.5 w-32" />
          <Skeleton className="h-9 w-full" />
        </div>
      ))}
    </div>
  );
}

export type SkeletonCardProps = { lines?: number; surface?: boolean; className?: string };

export function SkeletonCard({ lines = 2, surface = true, className }: SkeletonCardProps) {
  return (
    <div data-skeleton="card" aria-busy="true" className={cn("grid gap-3 p-4 sm:p-5", surface && "surface-panel rounded-2xl", className)}>
      <Skeleton className="h-5 w-40" />
      <SkeletonLines lines={lines} />
    </div>
  );
}

export type SkeletonWidgetProps = { rows?: number; className?: string };

export function SkeletonWidget({ rows = 2, className }: SkeletonWidgetProps) {
  const minHeight = rows * ROW_PIXELS + (rows - 1) * GAP_PIXELS;
  return (
    <div data-skeleton="widget" aria-busy="true" className={cn("surface-panel grid content-start gap-3 rounded-2xl p-4", className)} style={{ minHeight }}>
      <Skeleton className="h-4 w-28" />
      <Skeleton className="h-8 w-20" />
    </div>
  );
}

export type SkeletonPageProps = { sections?: number; className?: string };

export function SkeletonPage({ sections = 2, className }: SkeletonPageProps) {
  return (
    <div data-skeleton="page" aria-busy="true" className={cn("grid content-start gap-6", className)}>
      <div className="grid gap-2">
        <Skeleton className="h-7 w-56" />
        <Skeleton className="h-4 w-80 max-w-full" />
      </div>
      {Array.from({ length: sections }, (_, index) => (
        <SkeletonCard key={index} lines={3} />
      ))}
    </div>
  );
}
