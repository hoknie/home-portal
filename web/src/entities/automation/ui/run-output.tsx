"use client";

import { useTranslations } from "next-intl";
import { useLayoutEffect, useRef } from "react";

import type { Run } from "../model/schema";
import { terminalText } from "../model/terminal-text";

export const FOLLOW_SLACK_PIXELS = 16;

export type RunOutputProps = { label: string; output: Run["outcome"]["stdout"] };

export function RunOutput({ label, output }: RunOutputProps) {
  const t = useTranslations("automations");
  const block = useRef<HTMLPreElement>(null);
  const following = useRef(true);
  const text = terminalText(output.tail);
  useLayoutEffect(() => {
    const element = block.current;
    if (element && following.current) {
      element.scrollTop = element.scrollHeight;
    }
  }, [text]);
  const remember = () => {
    const element = block.current;
    if (element) {
      following.current = element.scrollHeight - element.scrollTop - element.clientHeight <= FOLLOW_SLACK_PIXELS;
    }
  };
  return (
    <div className="grid gap-1">
      <p className="text-sm font-medium">{label}</p>
      {output.truncated ? <p className="text-xs text-muted-foreground">{t("truncated", { bytes: output.bytes })}</p> : null}
      <pre
        ref={block}
        onScroll={remember}
        className="max-h-64 overflow-auto rounded-md border border-glass-edge bg-glass-tint p-3 font-mono text-xs whitespace-pre-wrap break-all"
      >
        {text === "" ? t("noOutput") : text}
      </pre>
    </div>
  );
}
