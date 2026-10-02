const NODE_SHADOW = "shadow-[0_1px_2px_rgb(0_0_0/0.06),0_4px_12px_-6px_rgb(0_0_0/0.12)]";

export const CANVAS = {
  node: `rounded-xl border border-border/70 bg-[var(--glass-overlay-solid)] text-card-foreground ${NODE_SHADOW} transition-[box-shadow,opacity] duration-200 hover:shadow-[0_1px_2px_rgb(0_0_0/0.08),0_8px_20px_-8px_rgb(0_0_0/0.2)]`,
  startNode: `rounded-2xl border border-border/70 bg-[color-mix(in_oklch,var(--color-primary)_7%,var(--glass-overlay-solid))] ${NODE_SHADOW}`,
  edgeLabel: "rounded-full border border-glass-edge bg-background shadow-xs",
  floatingChip: "rounded-full border border-glass-edge bg-background shadow-sm",
  fadeBottom: "bg-gradient-to-t from-[var(--glass-overlay-solid)] to-transparent",
  lifted: "shadow-lg",
  dotGrid: "bg-[radial-gradient(circle,var(--color-border)_1px,transparent_1px)] bg-[length:16px_16px]",
  dropDot: "shadow-[0_0_0_3px] shadow-primary/20",
  outcome: {
    running: "bg-[color-mix(in_oklch,var(--color-status-degraded)_8%,var(--glass-overlay-solid))]",
    succeeded: "bg-[color-mix(in_oklch,var(--color-status-up)_7%,var(--glass-overlay-solid))]",
    failed: "bg-[color-mix(in_oklch,var(--color-status-down)_9%,var(--glass-overlay-solid))]",
    "timed-out": "bg-[color-mix(in_oklch,var(--color-status-down)_9%,var(--glass-overlay-solid))]",
    stopped: "",
    skipped: "",
  } as Record<string, string>,
} as const;
