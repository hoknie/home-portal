"use client";

import type { CSSProperties, ReactNode } from "react";

import { PanelHandle } from "./panel-handle";
import { usePanelWidth } from "./use-panel-width";

export const PANEL_WIDTH_CLASS = "md:w-[clamp(20rem,var(--panel-width,26rem),70%)]";

export type PanelRowProps = { canvas: ReactNode; narrow: boolean; children: ReactNode };

export function PanelRow({ canvas, narrow, children }: PanelRowProps) {
  const width = usePanelWidth();
  return (
    <div className="flex h-[calc(100dvh-13rem)] min-h-[34rem] flex-col gap-3 md:flex-row" style={{ "--panel-width": `${width}px` } as CSSProperties} data-panel-row="">
      {canvas}
      {children && !narrow ? <PanelHandle /> : null}
      {children}
    </div>
  );
}
