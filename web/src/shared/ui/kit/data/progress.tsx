import { cn } from "@/shared/lib/cn";

export type ProgressTone = "neutral" | "warning" | "alarm";

const TONE: Record<ProgressTone, string> = {
  neutral: "bg-primary",
  warning: "bg-status-degraded",
  alarm: "bg-status-down",
};

export type ProgressProps = { percent: number; tone?: ProgressTone; label?: string; role?: "progressbar" | "meter"; className?: string };

export function Progress({ percent, tone = "neutral", label, role = "progressbar", className }: ProgressProps) {
  const filled = Math.min(100, Math.max(0, percent));
  return (
    <div
      data-slot="progress"
      role={role}
      aria-label={label}
      aria-valuemin={0}
      aria-valuemax={100}
      aria-valuenow={Math.round(filled)}
      className={cn("h-2 w-full overflow-hidden rounded-full bg-muted", className)}
    >
      <div className={cn("h-full rounded-full transition-[width] duration-200", TONE[tone])} style={{ width: `${filled}%` }} />
    </div>
  );
}
