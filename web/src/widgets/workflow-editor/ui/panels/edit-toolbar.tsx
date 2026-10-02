"use client";

import { cn } from "cn";
import { CircleAlert, CircleHelp, Redo2, TriangleAlert, Undo2 } from "lucide-react";
import { useTranslations } from "next-intl";

import { Button, Panel } from "@/shared/ui/kit";

export type EditToolbarProps = {
  canUndo: boolean;
  canRedo: boolean;
  errors: number;
  warnings: number;
  problemsOpen: boolean;
  saving: boolean;
  canSave?: boolean;
  onUndo: () => void;
  onRedo: () => void;
  onProblems: () => void;
  onLegend: () => void;
  onCancel: () => void;
};

export function EditToolbar({ canUndo, canRedo, errors, warnings, problemsOpen, saving, canSave = true, onUndo, onRedo, onProblems, onLegend, onCancel }: EditToolbarProps) {
  const t = useTranslations("workflowEditor.toolbar");
  const common = useTranslations("common");
  return (
    <Panel as="div" padding="none" className="flex flex-wrap items-center gap-1 rounded-xl p-1.5" role="toolbar" aria-label={t("label")}>
      <Button type="button" variant="ghost" size="icon" aria-label={t("undo")} title={t("undoKeys")} disabled={!canUndo} onClick={onUndo}>
        <Undo2 aria-hidden />
      </Button>
      <Button type="button" variant="ghost" size="icon" aria-label={t("redo")} title={t("redoKeys")} disabled={!canRedo} onClick={onRedo}>
        <Redo2 aria-hidden />
      </Button>
      <span className="mx-1 h-6 w-px bg-glass-edge" aria-hidden />
      <Button type="button" variant={problemsOpen ? "secondary" : "ghost"} size="sm" aria-pressed={problemsOpen} onClick={onProblems}>
        {errors > 0 ? <CircleAlert className="text-destructive" aria-hidden /> : <TriangleAlert className={cn(warnings > 0 ? "text-status-degraded" : "text-muted-foreground")} aria-hidden />}
        {t("problems", { errors, warnings })}
      </Button>
      <Button type="button" variant="ghost" size="icon" aria-label={t("legend")} onClick={onLegend}>
        <CircleHelp aria-hidden />
      </Button>
      <span className="ms-auto flex flex-wrap items-center gap-1">
        <Button type="button" variant="ghost" size="sm" onClick={onCancel} disabled={saving}>
          {common("cancel")}
        </Button>
        {canSave ? (
          <Button type="submit" size="sm" disabled={saving}>
            {saving ? common("saving") : errors > 0 ? t("saveWithErrors", { count: errors }) : common("save")}
          </Button>
        ) : null}
      </span>
    </Panel>
  );
}
