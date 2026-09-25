import { cn } from "@/shared/lib/cn";

export type MetricBarProps = {
  label: string;
  value: string;
  percent: number;
  tone?: "neutral" | "warning" | "alarm";
};

const TONE = {
  neutral: "bg-primary",
  warning: "bg-status-degraded",
  alarm: "bg-status-down",
} as const;

export function toneOf(percent: number): "neutral" | "warning" | "alarm" {
  if (percent >= 90) {
    return "alarm";
  }
  return percent >= 75 ? "warning" : "neutral";
}

export function MetricBar({ label, value, percent, tone }: MetricBarProps) {
  const filled = Math.min(100, Math.max(0, percent));
  const chosen = tone ?? toneOf(filled);
  return (
    <div className="grid gap-1.5">
      <div className="flex items-baseline justify-between gap-2 text-sm">
        <span className="text-muted-foreground">{label}</span>
        <span className="font-medium tabular-nums">{value}</span>
      </div>
      <div
        role="meter"
        aria-label={label}
        aria-valuenow={Math.round(filled)}
        aria-valuemin={0}
        aria-valuemax={100}
        className="h-2 overflow-hidden rounded-full bg-muted"
      >
        <div className={cn("h-full rounded-full transition-[width]", TONE[chosen])} style={{ width: `${filled}%` }} />
      </div>
    </div>
  );
}
