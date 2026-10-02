"use client";

import { X } from "lucide-react";
import { useTranslations } from "next-intl";
import type { ReactNode } from "react";

import { TraceTimeline } from "@/entities/automation";
import { START_ID, at, parsePath } from "@/entities/workflow";
import { cn } from "@/shared/lib/cn";
import { Button, Sheet, SheetContent, SheetDescription, SheetHeader, SheetTitle } from "@/shared/ui/primitives";

import { PANEL_WIDTH_CLASS } from "../resizing/panel-row";
import { useEditor } from "../../model/editor-context";
import { FALLBACK_ICON, GROUP_TONE, KIND_ICONS } from "../nodes/kind-style";
import { HeaderForm } from "./header-form";
import { Produces } from "./produces";
import { StepForm } from "./step-form";

function useBody() {
  const editor = useEditor();
  const help = useTranslations("workflowHelp");
  const t = useTranslations("workflowEditor");
  const selected = editor.selected;
  if (selected === START_ID) {
    return { title: t("settings"), description: t("settingsHint"), icon: null, content: <HeaderForm /> };
  }
  const path = selected ? parsePath(selected).path : [];
  const step = selected ? at(editor.draft.steps, path) : undefined;
  if (!step) {
    return null;
  }
  const kind = editor.kindOf(step.kind);
  const Icon = KIND_ICONS[step.kind] ?? FALLBACK_ICON;
  const run = editor.overlay.get(selected ?? "");
  const has = (key: string) => help.has(key as "kinds.if.name");
  return {
    title: has(`kinds.${step.kind}.name`) ? help(`kinds.${step.kind}.name` as "kinds.if.name") : step.kind,
    description: has(`kinds.${step.kind}.description`) ? help(`kinds.${step.kind}.description` as "kinds.if.description") : "",
    icon: (
      <span className={`grid size-8 place-items-center rounded-lg ${GROUP_TONE[kind?.group ?? "actions"].badge}`}>
        <Icon className="size-4" aria-hidden />
      </span>
    ),
    content: (
      <div className="grid gap-6">
        <StepForm key={selected} path={path} step={step} />
        <Produces step={step} />
        {run ? <TraceTimeline trace={{ entries: run.entries, dropped: 0, outputs: null }} indent={false} /> : null}
      </div>
    ),
  };
}

function Frame({ narrow, title, description, icon, children, onClose }: { narrow: boolean; title: string; description: string; icon: ReactNode; children: ReactNode; onClose: () => void }) {
  const t = useTranslations("workflowEditor");
  if (narrow) {
    return (
      <Sheet open onOpenChange={(open) => (open ? undefined : onClose())}>
        <SheetContent side="bottom" className="max-h-[85dvh] overflow-y-auto">
          <SheetHeader>
            <SheetTitle className="flex items-center gap-2">
              {icon}
              {title}
            </SheetTitle>
            <SheetDescription>{description}</SheetDescription>
          </SheetHeader>
          <div className="px-4 pb-6">{children}</div>
        </SheetContent>
      </Sheet>
    );
  }
  return (
    <aside aria-label={title} className={cn("glass-panel flex max-h-full min-h-0 w-full shrink-0 flex-col overflow-hidden rounded-2xl", PANEL_WIDTH_CLASS)}>
      <header className="flex items-start gap-3 border-b border-glass-edge p-4">
        {icon}
        <div className="grid min-w-0 flex-1 gap-0.5">
          <h2 className="text-base font-semibold">{title}</h2>
          <p className="text-xs text-muted-foreground">{description}</p>
        </div>
        <Button type="button" variant="ghost" size="icon" aria-label={t("closeInspector")} onClick={onClose}>
          <X aria-hidden />
        </Button>
      </header>
      <div className="min-h-0 flex-1 overflow-y-auto p-4">{children}</div>
    </aside>
  );
}

export function Inspector() {
  const editor = useEditor();
  const body = useBody();
  if (body === null) {
    return null;
  }
  return (
    <Frame narrow={editor.narrow} title={body.title} description={body.description} icon={body.icon} onClose={() => editor.select(null)}>
      {body.content}
    </Frame>
  );
}
