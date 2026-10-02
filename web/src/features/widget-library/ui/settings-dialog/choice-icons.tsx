import {
  AArrowDown,
  AArrowUp,
  AlignCenter,
  AlignCenterHorizontal,
  AlignEndHorizontal,
  AlignLeft,
  AlignRight,
  AlignStartHorizontal,
  ArrowDown,
  ArrowUp,
  Bold,
  CircleDashed,
  Eye,
  EyeOff,
  FoldHorizontal,
  Maximize,
  Minimize,
  MoveVertical,
  Scan,
  Type,
  UnfoldHorizontal,
} from "lucide-react";
import type { ReactNode } from "react";

import { cn } from "@/shared/lib/cn";
import { toneOf } from "@/shared/ui/kit";

export const ALIGN_ICONS: Record<"inherit" | "start" | "center" | "end", ReactNode> = {
  inherit: <CircleDashed aria-hidden />,
  start: <AlignLeft aria-hidden />,
  center: <AlignCenter aria-hidden />,
  end: <AlignRight aria-hidden />,
};

export const VALIGN_ICONS: Record<"inherit" | "start" | "center" | "end" | "stretch", ReactNode> = {
  inherit: <CircleDashed aria-hidden />,
  start: <AlignStartHorizontal aria-hidden />,
  center: <AlignCenterHorizontal aria-hidden />,
  end: <AlignEndHorizontal aria-hidden />,
  stretch: <MoveVertical aria-hidden />,
};

export const SIZE_ICONS: Record<string, ReactNode> = { small: <AArrowDown aria-hidden />, normal: <Type aria-hidden />, large: <AArrowUp aria-hidden /> };

export const WEIGHT_ICONS: Record<string, ReactNode> = { normal: <Type aria-hidden />, strong: <Bold aria-hidden /> };

export const GAP_ICONS: Record<string, ReactNode> = { small: <FoldHorizontal aria-hidden />, normal: <UnfoldHorizontal aria-hidden /> };

export const DIRECTION_ICONS: Record<"above" | "below", ReactNode> = { above: <ArrowUp aria-hidden />, below: <ArrowDown aria-hidden /> };

export const TITLE_ICONS: Record<"shown" | "hidden", ReactNode> = { shown: <Eye aria-hidden />, hidden: <EyeOff aria-hidden /> };

export const PADDING_ICONS: Record<"normal" | "compact" | "none", ReactNode> = { normal: <Maximize aria-hidden />, compact: <Minimize aria-hidden />, none: <Scan aria-hidden /> };

const STYLE_SHAPES: Record<string, string> = { primary: "bg-primary", secondary: "border border-foreground/40", ghost: "border border-dashed border-muted-foreground/40" };

export const STYLE_ICONS: Record<string, ReactNode> = Object.fromEntries(Object.entries(STYLE_SHAPES).map(([style, shape]) => [style, <span key={style} aria-hidden className={cn("h-3 w-5 rounded-sm", shape)} />]));

export function toneSwatch(tone: string): ReactNode {
  return <span aria-hidden className={cn("size-full rounded-full", tone === "neutral" ? "bg-muted-foreground" : toneOf(tone).fill)} />;
}
