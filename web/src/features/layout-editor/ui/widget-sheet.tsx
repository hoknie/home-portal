"use client";

import { useTranslations } from "next-intl";

import type { LayoutWidgetRequest } from "@/entities/dashboard";
import { FormField } from "@/shared/ui/form-field";
import { Button, Input, Label, Sheet, SheetContent, SheetDescription, SheetFooter, SheetHeader, SheetTitle, Switch } from "@/shared/ui/primitives";

import type { WidgetKind } from "../model/catalog";
import type { DraftWidget } from "../model/draft";

export type WidgetSheetProps = {
  widget: DraftWidget | null;
  kind: WidgetKind | null;
  title: string;
  environments: string[];
  onChange: (patch: Partial<LayoutWidgetRequest>) => void;
  onClose: () => void;
};

export function WidgetSheet({ widget, kind, title, environments, onChange, onClose }: WidgetSheetProps) {
  const t = useTranslations();
  const Editor = kind?.editor ?? null;
  const chosen = widget?.environments ?? [];
  const toggle = (name: string, on: boolean) => {
    const next = on ? [...chosen, name] : chosen.filter((candidate) => candidate !== name);
    onChange({ environments: next.length === 0 ? null : next });
  };
  return (
    <Sheet open={widget !== null} onOpenChange={(open) => (open ? undefined : onClose())}>
      <SheetContent className="w-full overflow-y-auto sm:max-w-lg" closeLabel={t("common.close")}>
        <SheetHeader>
          <SheetTitle>{title}</SheetTitle>
          <SheetDescription>{widget?.id ?? widget?.type}</SheetDescription>
        </SheetHeader>
        {widget ? (
          <div className="grid gap-5 px-4">
            <FormField id="widget-title" label={t("layoutEditor.widgetTitle")} hint={t("layoutEditor.widgetTitleHint")} optional>
              <Input id="widget-title" value={widget.title ?? ""} onChange={(event) => onChange({ title: event.target.value === "" ? null : event.target.value })} />
            </FormField>
            <fieldset className="grid gap-2">
              <legend className="text-sm font-medium">{t("layoutEditor.environments")}</legend>
              <p className="text-xs text-muted-foreground">{t("layoutEditor.environmentsHint")}</p>
              <div className="flex flex-wrap gap-3">
                {environments.map((name) => (
                  <label key={name} className="flex items-center gap-2 text-sm">
                    <input type="checkbox" checked={chosen.includes(name)} onChange={(event) => toggle(name, event.target.checked)} />
                    {name}
                  </label>
                ))}
              </div>
            </fieldset>
            <div className="flex items-center justify-between gap-4">
              <div className="grid gap-1">
                <Label htmlFor="widget-public">{t("layoutEditor.public")}</Label>
                <p className="text-xs text-muted-foreground">{t("layoutEditor.publicHint")}</p>
              </div>
              <Switch id="widget-public" checked={widget.public} onCheckedChange={(checked) => onChange({ public: checked })} />
            </div>
            <section className="grid gap-3">
              <h3 className="text-sm font-medium">{t("layoutEditor.settings")}</h3>
              {Editor ? (
                <Editor value={widget.settings} onChange={(settings) => onChange({ settings })} />
              ) : (
                <p className="text-sm text-muted-foreground">{t("layoutEditor.settingsInFile")}</p>
              )}
            </section>
          </div>
        ) : null}
        <SheetFooter className="flex-row justify-end">
          <Button onClick={onClose}>{t("layoutEditor.done")}</Button>
        </SheetFooter>
      </SheetContent>
    </Sheet>
  );
}
