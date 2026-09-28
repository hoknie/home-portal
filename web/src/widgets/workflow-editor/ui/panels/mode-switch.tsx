"use client";

import { cn } from "cn";
import { Eye, Pencil, Play } from "lucide-react";
import { useTranslations } from "next-intl";

import type { EditorMode } from "../../model/editor-context";

const SEGMENT = "flex items-center gap-1.5 rounded-md px-2.5 py-1 text-sm transition-colors disabled:opacity-50";

function Choice({ mode, option, onChange }: { mode: EditorMode; option: EditorMode; onChange: (mode: EditorMode) => void }) {
  const t = useTranslations("workflowEditor.toolbar.modes");
  const Icon = option === "edit" ? Pencil : Eye;
  return (
    <button
      type="button"
      role="radio"
      aria-checked={mode === option}
      onClick={() => onChange(option)}
      className={cn(SEGMENT, mode === option ? "bg-primary text-primary-foreground" : "text-muted-foreground hover:text-foreground")}
    >
      <Icon className="size-3.5" aria-hidden />
      {t(option)}
    </button>
  );
}

export type ModeSwitchProps = { mode: EditorMode; onChange: (mode: EditorMode) => void; running: boolean; onRun: () => void };

export function ModeSwitch({ mode, onChange, running, onRun }: ModeSwitchProps) {
  const t = useTranslations("workflowEditor.toolbar.modes");
  return (
    <div role="radiogroup" aria-label={t("label")} className="flex items-center gap-0.5 rounded-lg border border-glass-edge p-0.5">
      <Choice mode={mode} option="edit" onChange={onChange} />
      <button type="button" disabled={running} onClick={onRun} title={t("runHint")} className={cn(SEGMENT, "text-status-up hover:bg-status-up/10")}>
        <Play className="size-3.5" aria-hidden />
        {t("run")}
      </button>
      <Choice mode={mode} option="view" onChange={onChange} />
    </div>
  );
}
