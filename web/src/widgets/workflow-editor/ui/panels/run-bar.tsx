"use client";

import { EyeOff } from "lucide-react";
import { useTranslations } from "next-intl";

import { StopRunButton } from "@/features/stop-run";
import { OutcomeBadge, type Run, isActive } from "@/entities/automation";
import { Button } from "@/shared/ui/primitives";

export function RunBar({ run, title, onHide }: { run: Run; title: string; onHide: () => void }) {
  const t = useTranslations("workflowEditor.run");
  return (
    <div role="status" aria-label={t("title")} className="glass-panel flex flex-wrap items-center gap-2 rounded-xl px-3 py-2 text-sm shadow-md">
      <span className="font-medium">{t("title")}</span>
      <OutcomeBadge outcome={run.outcome.result} />
      {isActive(run) ? null : <span className="text-xs text-muted-foreground tabular-nums">{t("duration", { milliseconds: run.outcome.duration_milliseconds })}</span>}
      <span className="ms-auto flex items-center gap-1">
        {isActive(run) ? <StopRunButton run={run} title={title} /> : null}
        <Button type="button" variant="ghost" size="sm" onClick={onHide}>
          <EyeOff aria-hidden />
          {t("hide")}
        </Button>
      </span>
    </div>
  );
}
