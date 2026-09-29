"use client";

import { CanvasLoader } from "./canvas/canvas-loader";
import { Legend, rememberLegend } from "./panels/legend";

export type CanvasAreaProps = { legendOpen: boolean; onLegendClosed: () => void };

export function CanvasArea({ legendOpen, onLegendClosed }: CanvasAreaProps) {
  return (
    <div className="relative min-w-0 flex-1 overflow-hidden rounded-2xl border border-glass-edge bg-background/40">
      <CanvasLoader />
      {legendOpen ? (
        <div className="absolute top-3 left-3 z-20">
          <Legend
            onDismiss={() => {
              rememberLegend(true);
              onLegendClosed();
            }}
          />
        </div>
      ) : null}
    </div>
  );
}
