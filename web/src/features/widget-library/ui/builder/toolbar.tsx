"use client";

import { ArrowLeft, Redo2, Undo2 } from "lucide-react";
import { useTranslations } from "next-intl";

import { Button, Input, Panel } from "@/shared/ui/kit";

export type ToolbarProps = {
  title: string;
  placeholder: string;
  changed: boolean;
  saving: boolean;
  canSave: boolean;
  canUndo: boolean;
  canRedo: boolean;
  onTitle: (title: string) => void;
  onBack: () => void;
  onUndo: () => void;
  onRedo: () => void;
  onSave: () => void;
};

export function Toolbar(props: ToolbarProps) {
  const t = useTranslations("widgetBuilder");
  return (
    <Panel as="div" padding="none" className="flex flex-wrap items-center gap-3 rounded-xl px-4 py-3" data-builder-toolbar="">
      <Button type="button" variant="ghost" size="icon" onClick={props.onBack} aria-label={t("back")} title={t("back")}>
        <ArrowLeft aria-hidden />
      </Button>
      <Input aria-label={t("titleLabel")} placeholder={props.placeholder} className="h-9 max-w-72 flex-1 font-medium" value={props.title} onChange={(event) => props.onTitle(event.target.value)} />
      <p className="text-sm text-muted-foreground" role="status">
        {props.changed ? t("unsaved") : null}
      </p>
      <div className="ml-auto flex gap-2">
        <Button type="button" variant="ghost" size="icon" aria-label={t("undo")} title={t("undo")} disabled={!props.canUndo} onClick={props.onUndo}>
          <Undo2 aria-hidden />
        </Button>
        <Button type="button" variant="ghost" size="icon" aria-label={t("redo")} title={t("redo")} disabled={!props.canRedo} onClick={props.onRedo}>
          <Redo2 aria-hidden />
        </Button>
        <Button type="button" onClick={props.onSave} disabled={!props.canSave}>
          {props.saving ? t("saving") : t("save")}
        </Button>
      </div>
    </Panel>
  );
}
