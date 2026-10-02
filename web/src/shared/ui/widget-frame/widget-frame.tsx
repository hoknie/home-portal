"use client";

import { AlertTriangle, History } from "lucide-react";
import { useTranslations } from "next-intl";
import { type ReactNode, createContext, useContext, useId } from "react";

import { type Appearance, DEFAULT_APPEARANCE } from "@/shared/api";
import { cn } from "@/shared/lib/cn";
import { Heading, SkeletonLines, SURFACE } from "@/shared/ui/kit";

export type WidgetFrameProps = {
  title: string;
  stale?: boolean;
  problem?: string | null;
  loading?: boolean;
  appearance?: Appearance;
  fill?: boolean;
  children: ReactNode;
};

const SURFACES: Record<Appearance["surface"], string> = {
  card: `${SURFACE.panel} rounded-2xl`,
  plain: "",
  tinted: "widget-tint rounded-2xl",
  outline: "rounded-2xl border",
};

const PADDINGS: Record<Appearance["padding"], string> = {
  normal: "p-4 sm:p-5",
  compact: "p-3",
  none: "p-0",
};

const FrameSurface = createContext<Appearance["surface"]>("plain");

const FrameAlign = createContext<Appearance["align"]>("start");

const ALIGNS: Record<Appearance["align"], string> = {
  start: "",
  center: "items-center text-center",
  end: "items-end text-right",
};

export function useFrameSurface() {
  return useContext(FrameSurface);
}

export function useFrameAlign() {
  return useContext(FrameAlign);
}

export function WidgetFrame({ title, stale = false, problem = null, loading = false, appearance = DEFAULT_APPEARANCE, fill = false, children }: WidgetFrameProps) {
  const t = useTranslations("widgets");
  const heading = useId();
  const shown = appearance.title === "shown";
  const padded = appearance.surface === "plain" && appearance.padding === "normal" ? "p-0" : PADDINGS[appearance.padding];
  return (
    <section
      aria-labelledby={heading}
      data-surface={appearance.surface}
      data-accent={appearance.accent}
      data-align={appearance.align}
      className={cn("flex flex-col gap-3", SURFACES[appearance.surface], padded, fill && "h-full min-h-0", ALIGNS[appearance.align])}
    >
      <div className={cn("flex items-center gap-2", !shown && "sr-only")}>
        {appearance.accent !== "neutral" ? <span aria-hidden data-mark className="size-2 shrink-0 rounded-full bg-widget-accent" /> : null}
        <Heading level="section" as="h2" id={heading} className="text-base">
          {title}
        </Heading>
        {stale ? (
          <span
            data-stale="true"
            title={problem ?? undefined}
            className="inline-flex items-center gap-1 rounded-full border border-status-degraded/40 bg-status-degraded/10 px-2 py-0.5 text-xs text-status-degraded"
          >
            <History className="size-3" aria-hidden />
            {t("stale")}
          </span>
        ) : null}
      </div>
      <FrameSurface.Provider value={appearance.surface}>
        <FrameAlign.Provider value={appearance.align}>
          <div className={cn("w-full", fill && "min-h-0 flex-1 overflow-auto")}>{loading ? <SkeletonLines lines={3} /> : children}</div>
        </FrameAlign.Provider>
      </FrameSurface.Provider>
    </section>
  );
}

export function WidgetPanel({ className, children }: { className?: string; children: ReactNode }) {
  const surface = useFrameSurface();
  return <div className={cn(surface === "plain" && cn(SURFACE.panel, "rounded-xl p-4"), className)}>{children}</div>;
}

export function WidgetProblem({ message }: { message: string }) {
  const t = useTranslations("widgets");
  return (
    <div role="alert" className="flex items-start gap-3 rounded-xl border border-destructive/30 bg-destructive/5 p-4 text-sm">
      <AlertTriangle className="mt-0.5 size-4 shrink-0 text-destructive" aria-hidden />
      <div>
        <p className="font-medium">{t("failed")}</p>
        <p className="text-muted-foreground">{message}</p>
      </div>
    </div>
  );
}
