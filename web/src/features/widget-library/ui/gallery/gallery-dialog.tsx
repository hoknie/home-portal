"use client";

import { Sparkles } from "lucide-react";
import { useTranslations } from "next-intl";
import { useState } from "react";

import { enabledModules, useModules } from "@/entities/module";
import { cn } from "@/shared/lib/cn";
import { BareButton, Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle, Heading, Input } from "@/shared/ui/kit";

import { CUSTOM, type WidgetKind } from "../../model/catalog";
import { CUSTOM_TEMPLATES, type TemplateText } from "../../model/templates";

export type GalleryChoice = { type: string; settings: Record<string, unknown>; template: string | null };

export type GalleryDialogProps = { open: boolean; kinds: WidgetKind[]; onChoose: (choice: GalleryChoice) => void; onClose: () => void };

type Entry = { key: string; name: string; description: string; icon: WidgetKind["icon"]; off: string | null; choice: GalleryChoice };

function Tile({ entry, onChoose }: { entry: Entry; onChoose: (choice: GalleryChoice) => void }) {
  const Icon = entry.icon;
  return (
    <li>
      <BareButton
        disabled={entry.off !== null}
        title={entry.off ?? undefined}
        onClick={() => onChoose(entry.choice)}
        className={cn(
          "flex h-full w-full items-start gap-3 rounded-xl border p-3 text-left transition-colors outline-none focus-visible:ring-[3px] focus-visible:ring-ring/50",
          entry.off === null ? "hover:border-primary hover:bg-primary/5" : "cursor-not-allowed opacity-60",
        )}
        data-gallery={entry.key}
      >
        <Icon className="mt-0.5 size-5 shrink-0 text-primary" aria-hidden />
        <span className="grid gap-0.5">
          <span className="text-sm font-medium">{entry.name}</span>
          <span className="text-xs text-muted-foreground">{entry.off ?? entry.description}</span>
        </span>
      </BareButton>
    </li>
  );
}

export function GalleryDialog({ open, kinds, onChoose, onClose }: GalleryDialogProps) {
  const t = useTranslations("layoutEditor.gallery");
  const modules = useModules().data?.data;
  const on = modules === undefined ? null : enabledModules(modules);
  const [query, setQuery] = useState("");
  const offOf = (kind: WidgetKind) => (kind.module !== null && on !== null && !on.has(kind.module) ? t("moduleOff", { module: kind.module }) : null);
  const types: Entry[] = kinds.map((kind) => ({ key: kind.type, name: kind.title, description: kind.description, icon: kind.icon, off: offOf(kind), choice: { type: kind.type, settings: {}, template: null } }));
  const custom = kinds.find((kind) => kind.type === CUSTOM);
  const text = (key: TemplateText) => t(`templateTexts.${key}`);
  const templates: Entry[] = custom
    ? CUSTOM_TEMPLATES.map((template) => ({
        key: `template-${template.key}`,
        name: t(`templateNames.${template.key}`),
        description: custom.description,
        icon: Sparkles,
        off: offOf(custom),
        choice: { type: CUSTOM, settings: { blocks: template.blocks(text) }, template: template.key },
      }))
    : [];
  const wanted = query.trim().toLowerCase();
  const matches = (entry: Entry) => wanted === "" || `${entry.name} ${entry.description}`.toLowerCase().includes(wanted);
  const shownTypes = types.filter(matches);
  const shownTemplates = templates.filter(matches);
  return (
    <Dialog open={open} onOpenChange={(next) => (next ? undefined : onClose())}>
      <DialogContent className="max-h-[90dvh] overflow-y-auto sm:max-w-3xl">
        <DialogHeader>
          <DialogTitle>{t("title")}</DialogTitle>
          <DialogDescription>{t("description")}</DialogDescription>
        </DialogHeader>
        <Input type="search" aria-label={t("search")} placeholder={t("search")} value={query} onChange={(event) => setQuery(event.target.value)} autoFocus />
        {shownTypes.length > 0 ? (
          <section className="grid gap-2" aria-label={t("types")}>
            <Heading level="group" as="h3" className="text-sm font-medium">{t("types")}</Heading>
            <ul className="grid gap-2 sm:grid-cols-2">
              {shownTypes.map((entry) => (
                <Tile key={entry.key} entry={entry} onChoose={onChoose} />
              ))}
            </ul>
          </section>
        ) : null}
        {shownTemplates.length > 0 ? (
          <section className="grid gap-2" aria-label={t("templates")}>
            <Heading level="group" as="h3" className="text-sm font-medium">{t("templates")}</Heading>
            <ul className="grid gap-2 sm:grid-cols-2">
              {shownTemplates.map((entry) => (
                <Tile key={entry.key} entry={entry} onChoose={onChoose} />
              ))}
            </ul>
          </section>
        ) : null}
        {shownTypes.length + shownTemplates.length === 0 ? <p className="text-sm text-muted-foreground">{t("nothing")}</p> : null}
      </DialogContent>
    </Dialog>
  );
}
