import {
  Activity,
  CircleDashed,
  CircleStop,
  GitBranch,
  Globe,
  Hourglass,
  LogOut,
  type LucideIcon,
  Radar,
  Repeat,
  ScrollText,
  Shuffle,
  Send,
  SkipForward,
  Split,
  SquareFunction,
  Terminal,
  Variable,
  Workflow,
  Zap,
} from "lucide-react";

import type { KindGroup } from "@/entities/workflow";
import { CANVAS } from "@/shared/ui/kit";

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
  break: LogOut,
  continue: SkipForward,
};

export const FALLBACK_ICON = SquareFunction;

export const GROUP_TONE: Record<KindGroup, { badge: string }> = {
  flow: { badge: "bg-palette-violet/12 text-palette-violet" },
  data: { badge: "bg-palette-amber/12 text-palette-amber" },
  actions: { badge: "bg-palette-blue/12 text-palette-blue" },
};

export const OUTCOME_TINT: Record<string, string> = CANVAS.outcome;

export const OUTCOME_DOT: Record<string, string> = {
  running: "bg-status-degraded animate-pulse",
  succeeded: "bg-status-up",
  failed: "bg-status-down",
  "timed-out": "bg-status-down",
  stopped: "bg-muted-foreground",
  skipped: "bg-muted-foreground/50",
};
