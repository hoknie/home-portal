"use client";

import { PanelLeftClose, PanelLeftOpen, PanelRightClose, PanelRightOpen } from "lucide-react";
import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import { Splitter } from "@/shared/ui/splitter";
import { BareButton, Panel } from "@/shared/ui/kit";

export type SideColumnProps = {
  side: "left" | "right";
  title: string;
  collapsed: boolean;
  width: number;
  min: number;
  max: number;
  onToggle: () => void;
  onWidth: (width: number, done: boolean) => void;
  children: ReactNode;
};

export function SideColumn({ side, title, collapsed, width, min, max, onToggle, onWidth, children }: SideColumnProps) {
  const t = useTranslations("widgetBuilder.columns");
  const Open = side === "left" ? PanelLeftOpen : PanelRightOpen;
  if (collapsed) {
    return (
      <Panel as="div" padding="none" className="flex w-11 shrink-0 flex-col items-center gap-3 rounded-2xl py-2" data-side-column={side} data-collapsed="">
        <BareButton onClick={onToggle} aria-label={t("expand", { title })} title={t("expand", { title })} className="grid size-8 place-items-center rounded-lg text-muted-foreground hover:bg-muted hover:text-foreground">
          <Open className="size-4" aria-hidden />
        </BareButton>
        <span className="text-xs font-medium text-muted-foreground [writing-mode:vertical-rl]">{title}</span>
      </Panel>
    );
  }
  const splitter = <Splitter label={t("resize", { title })} orientation="vertical" value={width} min={min} max={max} direction={side === "left" ? 1 : -1} onChange={onWidth} className="-mx-1.5" />;
  return (
    <>
      {side === "right" ? splitter : null}
      <div className="flex min-h-0 shrink-0 flex-col gap-2" style={{ width }} data-side-column={side}>
        {children}
      </div>
      {side === "left" ? splitter : null}
    </>
  );
}

export function CollapseButton({ side, title, onToggle }: { side: "left" | "right"; title: string; onToggle: () => void }) {
  const t = useTranslations("widgetBuilder.columns");
  const Close = side === "left" ? PanelLeftClose : PanelRightClose;
  return (
    <BareButton onClick={onToggle} aria-label={t("collapse", { title })} title={t("collapse", { title })} className="grid size-7 place-items-center rounded-lg text-muted-foreground hover:bg-muted hover:text-foreground">
      <Close className="size-4" aria-hidden />
    </BareButton>
  );
}
