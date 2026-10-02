import type { ReactNode } from "react";

export type KvListProps = { children: ReactNode };

export type KvRowProps = { label: string; children: ReactNode };

export function KvList({ children }: KvListProps) {
  return <dl className="grid grid-cols-1 gap-x-6 gap-y-3 sm:grid-cols-[12rem_1fr]">{children}</dl>;
}

export function KvRow({ label, children }: KvRowProps) {
  return (
    <>
      <dt className="text-sm text-muted-foreground">{label}</dt>
      <dd className="text-sm font-medium break-words">{children}</dd>
    </>
  );
}
