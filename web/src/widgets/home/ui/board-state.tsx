import { SkeletonWidget } from "@/shared/ui/kit";

export function BoardSkeleton() {
  return (
    <div className="grid gap-4" data-skeleton="board" aria-busy="true">
      <SkeletonWidget rows={1} />
      <div className="grid gap-4 md:grid-cols-3">
        <SkeletonWidget rows={2} />
        <SkeletonWidget rows={2} />
        <SkeletonWidget rows={2} />
      </div>
    </div>
  );
}
