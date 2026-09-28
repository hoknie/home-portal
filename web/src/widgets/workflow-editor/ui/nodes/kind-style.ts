import {
  Activity,
  CircleDashed,
  CircleStop,
  GitBranch,
  Globe,
  Hourglass,
  type LucideIcon,
  Radar,
  Repeat,
  ScrollText,
  Shuffle,
  Send,
  Split,
  SquareFunction,
  Terminal,
  Variable,
  Workflow,
  Zap,
} from "lucide-react";

import type { KindGroup } from "@/entities/workflow";

export const KIND_ICONS: Record<string, LucideIcon> = {
  if: GitBranch,
  loop: Repeat,
  parallel: Split,
  workflow: Workflow,
  stop: CircleStop,
  set: Variable,
  wait: Hourglass,
  http: Globe,
  script: Terminal,
  notify: Send,
  log: ScrollText,
  transform: Shuffle,
  automation: Zap,
  probe: Radar,
  status: Activity,
  nothing: CircleDashed,
};

export const FALLBACK_ICON = SquareFunction;

export const GROUP_TONE: Record<KindGroup, { badge: string }> = {
  flow: { badge: "bg-violet-500/12 text-violet-600 dark:text-violet-300" },
  data: { badge: "bg-amber-500/12 text-amber-700 dark:text-amber-300" },
  actions: { badge: "bg-sky-500/12 text-sky-700 dark:text-sky-300" },
};

export const OUTCOME_TINT: Record<string, string> = {
  running: "bg-[color-mix(in_oklch,var(--color-status-degraded)_8%,var(--glass-overlay-solid))]",
  succeeded: "bg-[color-mix(in_oklch,var(--color-status-up)_7%,var(--glass-overlay-solid))]",
  failed: "bg-[color-mix(in_oklch,var(--color-status-down)_9%,var(--glass-overlay-solid))]",
  "timed-out": "bg-[color-mix(in_oklch,var(--color-status-down)_9%,var(--glass-overlay-solid))]",
  stopped: "",
  skipped: "",
};

export const OUTCOME_DOT: Record<string, string> = {
  running: "bg-status-degraded animate-pulse",
  succeeded: "bg-status-up",
  failed: "bg-status-down",
  "timed-out": "bg-status-down",
  stopped: "bg-muted-foreground",
  skipped: "bg-muted-foreground/50",
};
