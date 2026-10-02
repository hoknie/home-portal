"use client";

import { cn } from "cn";
import { useTranslations } from "next-intl";

import { useEditor } from "../../model/editor-context";
import { BareButton, Panel } from "@/shared/ui/kit";

const SEGMENT = "rounded-md px-2 py-0.5 text-xs transition-colors";

export function ValuesSwitch() {
  const t = useTranslations("workflowEditor.run");
  const editor = useEditor();
  return (
    <Panel as="div" padding="none" role="radiogroup" aria-label={t("valuesLabel")} className="flex items-center gap-0.5 rounded-lg p-0.5">
      {[true, false].map((values) => (
        <BareButton
          key={String(values)}
          role="radio"
          aria-checked={editor.showValues === values}
          onClick={() => editor.setShowValues(values)}
          className={cn(SEGMENT, editor.showValues === values ? "bg-primary text-primary-foreground" : "text-muted-foreground hover:text-foreground")}
        >
          {t(values ? "showValues" : "showTemplates")}
        </BareButton>
      ))}
    </Panel>
  );
}
