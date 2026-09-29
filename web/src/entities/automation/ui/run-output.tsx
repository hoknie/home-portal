"use client";

import { Copy } from "lucide-react";
import { useTranslations } from "next-intl";
import { useLayoutEffect, useRef } from "react";
import { toast } from "sonner";

import { Button } from "@/shared/ui/primitives";

import type { Run } from "../model/schema";
import { terminalText } from "../model/terminal-text";

export const FOLLOW_SLACK_PIXELS = 16;

const KIBIBYTE = 1024;

export type RunOutputProps = { label: string; output: Run["outcome"]["stdout"]; copy?: boolean; budget?: boolean };

export function keptKibibytes(tail: string) {
  return Math.max(1, Math.round(new TextEncoder().encode(tail).length / KIBIBYTE));
}

export function RunOutput({ label, output, copy = false, budget = false }: RunOutputProps) {
  const t = useTranslations("automations");
  const common = useTranslations("common");
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
  const copyText = async () => {
    try {
      await navigator.clipboard.writeText(text);
      toast.success(common("copied"));
    } catch {
      toast.error(common("copyFailed"));
    }
  };
  return (
    <div className="grid min-w-0 gap-1">
      <p className="text-sm font-medium">{label}</p>
      {output.truncated ? <p className="text-xs text-muted-foreground">{t("truncated", { kept: keptKibibytes(output.tail), bytes: output.bytes })}</p> : null}
      {budget ? <p className="text-xs text-muted-foreground">{t("budgetReached")}</p> : null}
      <div className="relative min-w-0">
        <pre
          ref={block}
          onScroll={remember}
          className="max-h-64 overflow-auto rounded-md border border-glass-edge bg-glass-tint p-3 font-mono text-xs whitespace-pre-wrap break-all"
        >
          {text === "" ? t("noOutput") : text}
        </pre>
        {copy && text !== "" ? (
          <Button type="button" variant="ghost" size="icon" className="absolute end-1 top-1 size-7" aria-label={common("copy")} onClick={() => void copyText()}>
            <Copy aria-hidden />
          </Button>
        ) : null}
      </div>
    </div>
  );
}
