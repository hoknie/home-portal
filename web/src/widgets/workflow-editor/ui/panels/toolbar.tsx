"use client";

import { cn } from "cn";
import { CircleAlert, CircleHelp, History, Redo2, TriangleAlert, Undo2 } from "lucide-react";
import Link from "next/link";
import { useTranslations } from "next-intl";

import { routes } from "@/shared/config";
import { Button } from "@/shared/ui/primitives";

import type { EditorMode } from "../../model/editor-context";
import { ModeSwitch } from "./mode-switch";

export type ToolbarProps = {
  canUndo: boolean;
  canRedo: boolean;
  errors: number;
  warnings: number;
  problemsOpen: boolean;
  runsOpen: boolean;
  saving: boolean;
  onUndo: () => void;
  onRedo: () => void;
  onProblems: () => void;
  onRuns: () => void;
  onLegend: () => void;
  onSaveAndRun: () => void;
  mode: EditorMode;
  onMode: (mode: EditorMode) => void;
};

export function Toolbar({ canUndo, canRedo, errors, warnings, problemsOpen, runsOpen, saving, onUndo, onRedo, onProblems, onRuns, onLegend, onSaveAndRun, mode, onMode }: ToolbarProps) {
  const viewing = mode === "view";
  const t = useTranslations("workflowEditor.toolbar");
  const common = useTranslations("common");
  return (
    <div className="glass-panel flex flex-wrap items-center gap-1 rounded-xl p-1.5" role="toolbar" aria-label={t("label")}>
      <ModeSwitch mode={mode} onChange={onMode} running={saving} onRun={onSaveAndRun} />
      <span className="mx-1 h-6 w-px bg-glass-edge" aria-hidden />
      <Button type="button" variant="ghost" size="icon" aria-label={t("undo")} title={t("undoKeys")} disabled={viewing || !canUndo} onClick={onUndo}>
        <Undo2 aria-hidden />
      </Button>
      <Button type="button" variant="ghost" size="icon" aria-label={t("redo")} title={t("redoKeys")} disabled={viewing || !canRedo} onClick={onRedo}>
        <Redo2 aria-hidden />
      </Button>
      <span className="mx-1 h-6 w-px bg-glass-edge" aria-hidden />
      <Button type="button" variant={problemsOpen ? "secondary" : "ghost"} size="sm" aria-expanded={problemsOpen} onClick={onProblems}>
        {errors > 0 ? <CircleAlert className="text-destructive" aria-hidden /> : <TriangleAlert className={cn(warnings > 0 ? "text-status-degraded" : "text-muted-foreground")} aria-hidden />}
        {t("problems", { errors, warnings })}
      </Button>
      <Button type="button" variant={runsOpen ? "secondary" : "ghost"} size="sm" aria-expanded={runsOpen} onClick={onRuns}>
        <History aria-hidden />
        {t("history")}
      </Button>
      <Button type="button" variant="ghost" size="icon" aria-label={t("legend")} onClick={onLegend}>
        <CircleHelp aria-hidden />
      </Button>
      <span className="ms-auto flex flex-wrap items-center gap-1">
        <Button asChild variant="ghost" size="sm">
          <Link href={routes.adminWorkflows}>{common("cancel")}</Link>
        </Button>
        <Button type="submit" size="sm" disabled={saving || viewing}>
          {saving ? common("saving") : errors > 0 ? t("saveWithErrors", { count: errors }) : common("save")}
        </Button>
      </span>
    </div>
  );
}
