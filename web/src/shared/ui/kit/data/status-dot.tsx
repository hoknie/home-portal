import { cn } from "@/shared/lib/cn";

export type StatusTone = "up" | "degraded" | "down" | "unreadable" | "unknown";

const TONE: Record<StatusTone, string> = {
  up: "bg-status-up",
  degraded: "bg-status-degraded",
  down: "bg-status-down",
  unreadable: "bg-status-unreadable",
  unknown: "bg-status-unknown",
};

export type StatusDotProps = { tone: StatusTone; label?: string; pulse?: boolean; className?: string };

export function StatusDot({ tone, label, pulse = false, className }: StatusDotProps) {
  return (
    <span
      data-slot="status-dot"
      data-tone={tone}
      role={label ? "img" : undefined}
      aria-label={label}
      aria-hidden={label ? undefined : true}
      className={cn("inline-block size-2 shrink-0 rounded-full", TONE[tone], pulse && "animate-pulse", className)}
    />
  );
}
