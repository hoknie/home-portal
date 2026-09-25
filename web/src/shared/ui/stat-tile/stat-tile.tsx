import type { ReactNode } from "react";

export type StatTileProps = {
  label: ReactNode;
  value: ReactNode;
  hint?: string;
};

export function StatTile({ label, value, hint }: StatTileProps) {
  return (
    <div className="rounded-xl border bg-card p-4">
      <p className="flex items-center gap-2 text-xs text-muted-foreground">{label}</p>
      <p className="mt-1 text-2xl font-semibold tabular-nums">{value}</p>
      {hint ? <p className="mt-1 text-xs text-muted-foreground">{hint}</p> : null}
    </div>
  );
}
